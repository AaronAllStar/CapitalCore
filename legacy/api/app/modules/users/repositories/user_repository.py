"""
Implementación concreta del repositorio de usuarios con SQLAlchemy.

Principio SOLID aplicado:
  - S (Single Responsibility): solo se encarga de acceso a datos de User
  - D (Dependency Inversion): implementa IUserRepository
"""
from __future__ import annotations
import uuid
from typing import Optional

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from app.modules.users.models.user import User
from app.modules.users.repositories import IUserRepository


def _user_query():
    """
    Query base que siempre carga eager-load de roles y subscription.

    Nota: UserRole.role ya tiene lazy="selectin" configurado en el mapeo
    imperativo de role.py, por lo que solo necesitamos cargar User.roles.
    """
    return (
        select(User)
        .options(selectinload(User.roles))
        .options(selectinload(User.subscription))
    )



class UserRepository(IUserRepository):
    """Implementación SQLAlchemy de IUserRepository."""

    def __init__(self, db: AsyncSession) -> None:
        self._db = db

    async def get_by_email(self, email: str) -> Optional[User]:
        result = await self._db.execute(
            _user_query().where(User.email == email)
        )
        return result.scalar_one_or_none()

    async def get_by_username(self, username: str) -> Optional[User]:
        result = await self._db.execute(
            select(User).where(User.username == username)
        )
        return result.scalar_one_or_none()

    async def get_by_id(self, user_id: uuid.UUID) -> Optional[User]:
        result = await self._db.execute(
            _user_query().where(User.id == user_id)
        )
        return result.scalar_one_or_none()

    async def add(self, user: User) -> None:
        self._db.add(user)
        await self._db.flush()

    async def refresh_relationships(self, user: User) -> None:
        await self._db.refresh(user, attribute_names=["roles", "subscription"])
