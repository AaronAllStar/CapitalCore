"""
Interfaces de repositorio para el módulo de usuarios.

Principio SOLID aplicado:
  - I (Interface Segregation): interfaces pequeñas y específicas
  - D (Dependency Inversion): el servicio depende de la abstracción, no de SQLAlchemy
"""
from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Optional
import uuid

from app.modules.users.models.user import User


class IUserRepository(ABC):
    """Abstracción del repositorio de usuarios."""

    @abstractmethod
    async def get_by_email(self, email: str) -> Optional[User]:
        """Busca un usuario por email, con roles y subscription cargados."""
        ...

    @abstractmethod
    async def get_by_username(self, username: str) -> Optional[User]:
        """Busca un usuario por username."""
        ...

    @abstractmethod
    async def get_by_id(self, user_id: uuid.UUID) -> Optional[User]:
        """Busca un usuario por ID, con roles y subscription cargados."""
        ...

    @abstractmethod
    async def add(self, user: User) -> None:
        """Agrega un usuario a la sesión (sin commit)."""
        ...

    @abstractmethod
    async def refresh_relationships(self, user: User) -> None:
        """Recarga las relaciones de roles y subscription en el contexto async."""
        ...
