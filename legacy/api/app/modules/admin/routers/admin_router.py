import uuid
from fastapi import APIRouter, Depends, Query
from sqlalchemy import select, func, desc
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import require_admin
from app.modules.users.models.user import User
from app.modules.strategies.models.strategy import Strategy
from app.modules.backtest.models.backtest_job import BacktestJob
from app.modules.tournaments.models.tournament import Tournament

from app.modules.rbac.services.role_service import RoleService
from app.core.config import get_settings

router = APIRouter()
_settings = get_settings()


@router.post("/seed-roles", include_in_schema=False)
async def seed_roles(db: AsyncSession = Depends(get_session)):
    """
    Inicializa los roles y permisos por defecto.
    Solo disponible en entorno de desarrollo.
    """
    if _settings.ENVIRONMENT != "development":
        from fastapi import HTTPException
        raise HTTPException(status_code=403, detail="Only available in development")
    role_svc = RoleService(db)
    await role_svc.seed_defaults()
    return {"message": "Roles seeded successfully"}


@router.get("/stats")
async def platform_stats(
    admin: User = Depends(require_admin),
    db: AsyncSession = Depends(get_session),
):
    user_count = (await db.execute(select(func.count()).select_from(User))).scalar()
    strategy_count = (await db.execute(select(func.count()).select_from(Strategy))).scalar()
    backtest_count = (await db.execute(select(func.count()).select_from(BacktestJob))).scalar()
    tournament_count = (await db.execute(select(func.count()).select_from(Tournament))).scalar()

    return {
        "users": user_count,
        "strategies": strategy_count,
        "backtests": backtest_count,
        "tournaments": tournament_count,
    }


@router.get("/users")
async def list_users(
    admin: User = Depends(require_admin),
    db: AsyncSession = Depends(get_session),
    limit: int = Query(50, le=200),
    offset: int = Query(0, ge=0),
):
    result = await db.execute(
        select(User)
        .order_by(desc(User.created_at))
        .limit(limit)
        .offset(offset)
    )
    users = list(result.scalars().all())
    return [
        {
            "id": str(u.id),
            "email": u.email,
            "username": u.username,
            "plan": u.plan,
            "is_active": u.is_active,
            "is_banned": u.is_banned,
            "rating": u.rating,
            "created_at": str(u.created_at),
        }
        for u in users
    ]


@router.post("/users/{user_id}/ban")
async def ban_user(
    user_id: uuid.UUID,
    reason: str = "Violation of terms",
    admin: User = Depends(require_admin),
    db: AsyncSession = Depends(get_session),
):
    user = await db.get(User, user_id)
    if not user:
        return {"error": "User not found"}
    user.is_banned = True
    user.ban_reason = reason
    await db.flush()
    return {"message": f"User {user.username} banned", "reason": reason}


@router.post("/users/{user_id}/unban")
async def unban_user(
    user_id: uuid.UUID,
    admin: User = Depends(require_admin),
    db: AsyncSession = Depends(get_session),
):
    user = await db.get(User, user_id)
    if not user:
        return {"error": "User not found"}
    user.is_banned = False
    user.ban_reason = None
    await db.flush()
    return {"message": f"User {user.username} unbanned"}
