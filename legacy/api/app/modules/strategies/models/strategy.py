import uuid
import enum
from datetime import datetime
from sqlalchemy import String, Boolean, Integer, Text, ForeignKey, Enum as SAEnum, Index
from sqlalchemy.dialects.postgresql import UUID, JSONB, ARRAY
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class StrategyStatus(str, enum.Enum):
    DRAFT = "draft"
    ACTIVE = "active"
    ARCHIVED = "archived"
    SUSPENDED = "suspended"


class StrategySource(str, enum.Enum):
    MANUAL = "manual"
    AI = "ai"
    TEMPLATE = "template"
    IMPORTED = "imported"


class Strategy(Base):
    __tablename__ = "strategies"

    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False, index=True
    )
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    version: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    status: Mapped[str] = mapped_column(
        SAEnum(StrategyStatus, native_enum=False), default=StrategyStatus.DRAFT, nullable=False
    )
    is_public: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    tags: Mapped[list[str] | None] = mapped_column(ARRAY(Text))
    rules: Mapped[dict] = mapped_column(JSONB, nullable=False)

    # Origin tracking
    source: Mapped[str] = mapped_column(
        SAEnum(StrategySource, native_enum=False), default=StrategySource.MANUAL, nullable=False
    )
    template_id: Mapped[str | None] = mapped_column(String(100))
    natural_input: Mapped[str | None] = mapped_column(Text)
    ai_model: Mapped[str | None] = mapped_column(String(50))

    # Marketplace
    is_for_sale: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    price: Mapped[float | None] = mapped_column(nullable=True)

    # Stats (denormalized for fast reads)
    total_backtests: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    total_wins: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    best_sharpe: Mapped[float | None] = mapped_column(nullable=True)
    best_pnl_pct: Mapped[float | None] = mapped_column(nullable=True)
    win_rate: Mapped[float | None] = mapped_column(nullable=True)

    # Relationships
    user = relationship("User", back_populates="strategies", lazy="joined")
    versions = relationship("StrategyVersion", back_populates="strategy", lazy="selectin", cascade="all, delete-orphan")

    __table_args__ = (
        Index("ix_strategy_user_status", "user_id", "status"),
        Index("ix_strategy_public", "is_public", "status"),
    )

    def __repr__(self) -> str:
        return f"<Strategy {self.name} v{self.version} ({self.id})>"


class StrategyVersion(Base):
    __tablename__ = "strategy_versions"

    strategy_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("strategies.id", ondelete="CASCADE"), nullable=False, index=True
    )
    version: Mapped[int] = mapped_column(Integer, nullable=False)
    rules: Mapped[dict] = mapped_column(JSONB, nullable=False)
    changelog: Mapped[str | None] = mapped_column(Text)

    strategy = relationship("Strategy", back_populates="versions")

    __table_args__ = (
        Index("ix_strategy_version_unique", "strategy_id", "version", unique=True),
    )


class StrategyCopy(Base):
    __tablename__ = "strategy_copies"

    original_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("strategies.id", ondelete="CASCADE"), nullable=False
    )
    copied_by: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
