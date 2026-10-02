"""FastAPI dependency injection."""
from collections.abc import AsyncGenerator
from fastapi import Depends, Header, Request
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.core.cache.redis import get_redis, CacheService
from app.core.security.jwt import decode_access_token
from app.core.exceptions import UnauthorizedError, ForbiddenError
from app.modules.users.models.user import User
from app.modules.rbac.models.role import RoleEnum
import redis.asyncio as aioredis


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    async for session in get_session():
        yield session


async def get_cache() -> CacheService:
    r = await get_redis()
    return CacheService(r)


async def get_current_user(
    authorization: str = Header(..., alias="Authorization"),
    db: AsyncSession = Depends(get_db),
) -> User:
    if not authorization.startswith("Bearer "):
        raise UnauthorizedError("Invalid authorization header format")

    token = authorization.split(" ", 1)[1]
    payload = decode_access_token(token)

    if payload.get("type") != "access":
        raise UnauthorizedError("Invalid token type")

    user_id = payload.get("sub")
    if not user_id:
        raise UnauthorizedError("Token missing subject")

    user = await db.get(User, user_id)
    if not user:
        raise UnauthorizedError("User not found")
    if not user.is_active:
        raise UnauthorizedError("Account is deactivated")

    return user


async def get_optional_user(
    authorization: str | None = Header(None, alias="Authorization"),
    db: AsyncSession = Depends(get_db),
) -> User | None:
    if not authorization:
        return None
    try:
        return await get_current_user(authorization, db)
    except (UnauthorizedError, Exception):
        return None


def require_role(*roles: RoleEnum):
    """Dependency factory that checks user has one of the required roles."""

    async def _check(
        user: User = Depends(get_current_user),
        db: AsyncSession = Depends(get_db),
    ) -> User:
        from app.modules.rbac.services.role_service import RoleService

        svc = RoleService(db)
        user_roles = await svc.get_user_roles(user.id)
        role_names = {r.name for r in user_roles}

        if not any(r.value in role_names for r in roles):
            raise ForbiddenError(
                f"Required role: {', '.join(r.value for r in roles)}"
            )
        return user

    return _check


async def require_admin(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_db),
) -> User:
    from app.modules.rbac.services.role_service import RoleService

    svc = RoleService(db)
    if not await svc.has_role(user.id, RoleEnum.ADMIN):
        raise ForbiddenError("Admin access required")
    return user
