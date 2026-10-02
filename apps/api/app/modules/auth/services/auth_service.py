"""
AuthService — lógica de negocio de autenticación y perfil de usuario.

Principios SOLID aplicados:
  - S (Single Responsibility): solo coordina autenticación, delega tokens a TokenService
    y acceso a datos a UserRepository
  - O (Open/Closed): la lógica no cambia si se reemplaza la BD o el proveedor de tokens
  - D (Dependency Inversion): depende de IUserRepository (abstracción), no de SQLAlchemy
"""
from __future__ import annotations
import uuid

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.exceptions import (
    UnauthorizedError,
    ConflictError,
    NotFoundError,
    BusinessRuleError,
)
from app.core.security.password import hash_password, verify_password
from app.modules.users.models.user import User
from app.modules.rbac.models.role import Role, UserRole, RoleEnum
from app.modules.users.repositories import IUserRepository
from app.modules.users.repositories.user_repository import UserRepository
from app.modules.auth.services.token_service import TokenService
from app.modules.auth.schemas.auth_schema import (
    RegisterRequest,
    LoginRequest,
    UpdateProfileRequest,
    ChangePasswordRequest,
    AuthResponse,
    TokenResponse,
    UserProfile,
)


class AuthService:
    """
    Coordina el flujo de autenticación.

    Depende de:
      - IUserRepository: acceso a datos de usuarios (DIP)
      - TokenService: ciclo de vida de tokens (SRP)
    """

    def __init__(
        self,
        db: AsyncSession,
        user_repo: IUserRepository | None = None,
        token_svc: TokenService | None = None,
    ) -> None:
        self._db = db
        # Permite inyección de mocks en tests (DIP + O/C)
        self._user_repo: IUserRepository = user_repo or UserRepository(db)
        self._token_svc: TokenService = token_svc or TokenService(db)

    # ─── Register ────────────────────────────────────────────────────────────

    async def register(
        self,
        data: RegisterRequest,
        user_agent: str | None = None,
        ip: str | None = None,
    ) -> AuthResponse:
        # Check duplicates
        if await self._user_repo.get_by_email(data.email):
            raise ConflictError("Email already registered")
        if await self._user_repo.get_by_username(data.username):
            raise ConflictError("Username already taken")

        # Create user
        user = User(
            email=data.email,
            username=data.username,
            password_hash=hash_password(data.password),
            display_name=data.display_name,
        )
        await self._user_repo.add(user)

        # Assign default role
        await self._assign_default_role(user)

        # Reload relationships so UserProfile.model_validate works correctly
        await self._user_repo.refresh_relationships(user)

        tokens = await self._token_svc.create_pair(user.id, user_agent, ip)
        return AuthResponse(
            user=UserProfile.model_validate(user),
            tokens=tokens,
        )

    # ─── Login ───────────────────────────────────────────────────────────────

    async def login(
        self,
        data: LoginRequest,
        user_agent: str | None = None,
        ip: str | None = None,
    ) -> AuthResponse:
        # UserRepository always eager-loads roles + subscription (BUG 3 fix)
        user = await self._user_repo.get_by_email(data.email)

        if not user or not verify_password(data.password, user.password_hash):
            raise UnauthorizedError("Invalid email or password")

        if not user.is_active:
            raise UnauthorizedError("Account is deactivated")

        if user.is_banned:
            raise BusinessRuleError(
                f"Account banned: {user.ban_reason or 'No reason provided'}"
            )

        tokens = await self._token_svc.create_pair(user.id, user_agent, ip)
        return AuthResponse(
            user=UserProfile.model_validate(user),
            tokens=tokens,
        )

    # ─── Token management ────────────────────────────────────────────────────

    async def refresh(self, refresh_token: str) -> TokenResponse:
        return await self._token_svc.rotate(refresh_token)

    async def logout(self, refresh_token: str) -> None:
        await self._token_svc.revoke(refresh_token)

    async def logout_all(self, user_id: uuid.UUID) -> int:
        return await self._token_svc.revoke_all(user_id)

    # ─── Profile ─────────────────────────────────────────────────────────────

    async def get_profile(self, user_id: uuid.UUID) -> UserProfile:
        user = await self._user_repo.get_by_id(user_id)
        if not user:
            raise NotFoundError("User")
        return UserProfile.model_validate(user)

    async def update_profile(self, user: User, data: UpdateProfileRequest) -> UserProfile:
        if data.display_name is not None:
            user.display_name = data.display_name
        if data.avatar_url is not None:
            user.avatar_url = data.avatar_url
        if data.bio is not None:
            user.bio = data.bio
        await self._db.flush()
        # Reload so the @property `plan` has access to subscription
        await self._user_repo.refresh_relationships(user)
        return UserProfile.model_validate(user)

    async def change_password(self, user: User, data: ChangePasswordRequest) -> None:
        if not verify_password(data.current_password, user.password_hash):
            raise UnauthorizedError("Current password is incorrect")
        user.password_hash = hash_password(data.new_password)
        # Revoke all sessions on password change
        await self._token_svc.revoke_all(user.id)
        await self._db.flush()

    # ─── Private helpers ─────────────────────────────────────────────────────

    async def _assign_default_role(self, user: User) -> None:
        """Asigna el rol 'user' por defecto al nuevo usuario registrado."""
        result = await self._db.execute(
            select(Role).where(Role.name == RoleEnum.USER.value)
        )
        default_role = result.scalar_one_or_none()
        if default_role:
            self._db.add(UserRole(user_id=user.id, role_id=default_role.id))
            await self._db.flush()
