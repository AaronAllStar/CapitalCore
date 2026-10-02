"""
TokenService — gestión del ciclo de vida de tokens JWT y refresh tokens.

Principio SOLID aplicado:
  - S (Single Responsibility): solo maneja tokens, nada de usuarios ni reglas de negocio
  - O (Open/Closed): extendible para otros tipos de token (e.g. magic link) sin modificar la clase
"""
from __future__ import annotations
import hashlib
import uuid
from datetime import datetime, timedelta, timezone

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import get_settings
from app.core.exceptions import UnauthorizedError
from app.core.security.jwt import (
    create_access_token,
    create_refresh_token,
    decode_refresh_token,
)
from app.modules.users.models.user import RefreshToken
from app.modules.auth.schemas.auth_schema import TokenResponse

settings = get_settings()


class TokenService:
    """
    Responsabilidad única: crear, rotar y revocar tokens de autenticación.
    """

    def __init__(self, db: AsyncSession) -> None:
        self._db = db

    async def create_pair(
        self,
        user_id: uuid.UUID,
        user_agent: str | None = None,
        ip: str | None = None,
    ) -> TokenResponse:
        """Genera un par access/refresh token y persiste el refresh token hasheado."""
        access = create_access_token(str(user_id))
        refresh = create_refresh_token(str(user_id))

        token_hash = self._hash(refresh)
        expires_at = (
            datetime.now(timezone.utc) + timedelta(days=settings.REFRESH_TOKEN_EXPIRE_DAYS)
        ).replace(tzinfo=None)

        self._db.add(
            RefreshToken(
                user_id=user_id,
                token_hash=token_hash,
                expires_at=expires_at,
                user_agent=user_agent,
                ip_address=ip,
            )
        )
        await self._db.flush()

        return TokenResponse(
            access_token=access,
            refresh_token=refresh,
            expires_in=settings.ACCESS_TOKEN_EXPIRE_MINUTES * 60,
        )

    async def rotate(self, refresh_token: str) -> TokenResponse:
        """Valida el refresh token, lo revoca y emite un nuevo par."""
        payload = decode_refresh_token(refresh_token)
        user_id = payload.get("sub")

        token_hash = self._hash(refresh_token)
        result = await self._db.execute(
            select(RefreshToken).where(
                RefreshToken.token_hash == token_hash,
                RefreshToken.user_id == uuid.UUID(user_id),
                RefreshToken.revoked == False,  # noqa: E712
                RefreshToken.expires_at > datetime.now(timezone.utc).replace(tzinfo=None),
            )
        )
        stored = result.scalar_one_or_none()
        if not stored:
            raise UnauthorizedError("Invalid or expired refresh token")

        # Revoke old token before issuing new one (rotation)
        stored.revoked = True
        await self._db.flush()

        return await self.create_pair(uuid.UUID(user_id))

    async def revoke(self, refresh_token: str) -> None:
        """Revoca un refresh token específico (logout)."""
        token_hash = self._hash(refresh_token)
        result = await self._db.execute(
            select(RefreshToken).where(RefreshToken.token_hash == token_hash)
        )
        stored = result.scalar_one_or_none()
        if stored:
            stored.revoked = True
            await self._db.flush()

    async def revoke_all(self, user_id: uuid.UUID) -> int:
        """Revoca todos los refresh tokens activos del usuario (logout-all)."""
        result = await self._db.execute(
            select(RefreshToken).where(
                RefreshToken.user_id == user_id,
                RefreshToken.revoked == False,  # noqa: E712
            )
        )
        tokens = result.scalars().all()
        for t in tokens:
            t.revoked = True
        await self._db.flush()
        return len(tokens)

    @staticmethod
    def _hash(token: str) -> str:
        return hashlib.sha256(token.encode()).hexdigest()
