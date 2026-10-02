"""
Módulo RBAC — Roles, Permisos y asociación User↔Role.

UserRole usa mapeo imperativo sobre Base.registry para evitar
heredar columnas de Base (id, created_at, updated_at).
Post-migración a1b2c3d4e5f6, la tabla user_roles tiene PK (user_id, role_id).
"""
import uuid
import enum
from sqlalchemy import String, ForeignKey, Table, Column
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class RoleEnum(str, enum.Enum):
    USER = "user"
    MODERATOR = "moderator"
    ADMIN = "admin"
    SYSTEM = "system"


class PermissionEnum(str, enum.Enum):
    # Strategies
    STRATEGY_CREATE = "strategy:create"
    STRATEGY_READ = "strategy:read"
    STRATEGY_UPDATE = "strategy:update"
    STRATEGY_DELETE = "strategy:delete"
    STRATEGY_PUBLISH = "strategy:publish"

    # Backtests
    BACKTEST_CREATE = "backtest:create"
    BACKTEST_READ = "backtest:read"
    BACKTEST_CANCEL = "backtest:cancel"

    # Tournaments
    TOURNAMENT_JOIN = "tournament:join"
    TOURNAMENT_CREATE = "tournament:create"
    TOURNAMENT_MANAGE = "tournament:manage"

    # Marketplace
    MARKETPLACE_BUY = "marketplace:buy"
    MARKETPLACE_SELL = "marketplace:sell"

    # Admin
    ADMIN_USERS = "admin:users"
    ADMIN_STRATEGIES = "admin:strategies"
    ADMIN_TOURNAMENTS = "admin:tournaments"
    ADMIN_BILLING = "admin:billing"
    ADMIN_ANALYTICS = "admin:analytics"


# ─── Tablas de asociación puras ───────────────────────────────────────────────

role_permissions_table = Table(
    "role_permissions",
    Base.metadata,
    Column("role_id", UUID(as_uuid=True), ForeignKey("roles.id", ondelete="CASCADE"), primary_key=True),
    Column("permission_id", UUID(as_uuid=True), ForeignKey("permissions.id", ondelete="CASCADE"), primary_key=True),
)

# Tabla user_roles post-migración: PK compuesta (user_id, role_id), sin columna id
_user_roles_table = Table(
    "user_roles",
    Base.metadata,
    Column("user_id", UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), primary_key=True),
    Column("role_id", UUID(as_uuid=True), ForeignKey("roles.id", ondelete="CASCADE"), primary_key=True),
)


# ─── ORM Models ──────────────────────────────────────────────────────────────

class Role(Base):
    __tablename__ = "roles"

    name: Mapped[str] = mapped_column(String(50), unique=True, nullable=False, index=True)
    description: Mapped[str | None] = mapped_column(String(255))
    permissions = relationship("Permission", secondary=role_permissions_table, lazy="selectin")


class Permission(Base):
    __tablename__ = "permissions"

    name: Mapped[str] = mapped_column(String(100), unique=True, nullable=False, index=True)
    description: Mapped[str | None] = mapped_column(String(255))


class UserRole:
    """
    Clase ORM para user_roles.

    Mapeada imperativamente usando Base.registry para compartir el mismo
    registry que User y Role, lo que permite relaciones ORM entre ellos.
    PK compuesta (user_id, role_id) — sin columna id propia.
    """
    def __init__(self, user_id: uuid.UUID, role_id: uuid.UUID) -> None:
        self.user_id = user_id
        self.role_id = role_id

    def __repr__(self) -> str:
        return f"<UserRole user={self.user_id} role={self.role_id}>"


# Registrar UserRole en el mismo registry de Base usando mapeo imperativo.
# Esto permite relacionarlo con User y Role sin heredar las columnas de Base.
Base.registry.map_imperatively(
    UserRole,
    _user_roles_table,
    properties={
        "user": relationship("User", back_populates="roles", foreign_keys=[_user_roles_table.c.user_id]),
        "role": relationship("Role", lazy="selectin", foreign_keys=[_user_roles_table.c.role_id]),
    },
)
