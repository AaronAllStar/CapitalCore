import uuid
from datetime import datetime
from sqlalchemy import String, Boolean, Integer, Text
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class User(Base):
    __tablename__ = "users"

    email: Mapped[str] = mapped_column(String(255), unique=True, nullable=False, index=True)
    username: Mapped[str] = mapped_column(String(50), unique=True, nullable=False, index=True)
    password_hash: Mapped[str] = mapped_column(String(255), nullable=False)
    display_name: Mapped[str | None] = mapped_column(String(100))
    avatar_url: Mapped[str | None] = mapped_column(Text)
    bio: Mapped[str | None] = mapped_column(Text)
    email_verified: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)
    is_banned: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    ban_reason: Mapped[str | None] = mapped_column(Text)

    # Stats (denormalized for fast reads)
    rating: Mapped[int] = mapped_column(Integer, default=1200, nullable=False)
    peak_rating: Mapped[int] = mapped_column(Integer, default=1200, nullable=False)
    total_wins: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    total_losses: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    total_tournaments: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    total_backtests: Mapped[int] = mapped_column(Integer, default=0, nullable=False)

    # Relationships — roles usa la tabla de asociación pura (sin columna `id`)
    # La definición completa de foreign_keys está en role.py (mapeo imperativo de UserRole)
    roles = relationship(
        "UserRole",
        back_populates="user",
        lazy="selectin",
        cascade="all, delete-orphan",
    )
    strategies = relationship("Strategy", back_populates="user", lazy="selectin", cascade="all, delete-orphan")
    subscription = relationship("Subscription", back_populates="user", lazy="selectin", uselist=False)



    @property
    def plan(self) -> str:
        if self.subscription and self.subscription.is_active:
            return self.subscription.plan
        return "free"

    @property
    def is_admin(self) -> bool:
        return any(r.role.name == "admin" for r in self.roles)

    def __repr__(self) -> str:
        return f"<User {self.username} ({self.id})>"


class RefreshToken(Base):
    __tablename__ = "refresh_tokens"

    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), nullable=False, index=True
    )
    token_hash: Mapped[str] = mapped_column(String(255), nullable=False, index=True)
    expires_at: Mapped[datetime] = mapped_column(nullable=False)
    revoked: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    user_agent: Mapped[str | None] = mapped_column(Text)
    ip_address: Mapped[str | None] = mapped_column(String(45))
