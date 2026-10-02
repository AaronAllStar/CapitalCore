import uuid
import enum
from datetime import datetime
from sqlalchemy import String, Integer, Text, ForeignKey, Enum as SAEnum, Index
from sqlalchemy import Numeric
from sqlalchemy.dialects.postgresql import UUID, JSONB
from sqlalchemy.orm import Mapped, mapped_column
from app.core.db.base import Base


class BacktestStatus(str, enum.Enum):
    QUEUED = "queued"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class BacktestPriority(str, enum.Enum):
    LOW = "low"
    NORMAL = "normal"
    HIGH = "high"
    TOURNAMENT = "tournament"


class BacktestJob(Base):
    __tablename__ = "backtest_jobs"

    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id"), nullable=False, index=True
    )
    strategy_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("strategies.id", ondelete="CASCADE"), nullable=False
    )
    strategy_version: Mapped[int] = mapped_column(Integer, nullable=False)
    strategy_snapshot: Mapped[dict] = mapped_column(JSONB, nullable=False)

    # Execution config
    status: Mapped[str] = mapped_column(
        SAEnum(BacktestStatus, native_enum=False), default=BacktestStatus.QUEUED, nullable=False
    )
    priority: Mapped[str] = mapped_column(
        SAEnum(BacktestPriority, native_enum=False), default=BacktestPriority.NORMAL, nullable=False
    )
    symbol: Mapped[str] = mapped_column(String(20), nullable=False)
    timeframe: Mapped[str] = mapped_column(String(10), nullable=False)
    start_date: Mapped[datetime] = mapped_column(nullable=False)
    end_date: Mapped[datetime] = mapped_column(nullable=False)
    initial_capital: Mapped[float] = mapped_column(Numeric(20, 8), default=10000)

    # Results
    results: Mapped[dict | None] = mapped_column(JSONB)
    equity_curve: Mapped[list | None] = mapped_column(JSONB)
    trades: Mapped[list | None] = mapped_column(JSONB)

    # Metadata
    error_message: Mapped[str | None] = mapped_column(Text)
    execution_time_ms: Mapped[int | None] = mapped_column(Integer)
    candle_count: Mapped[int | None] = mapped_column(Integer)
    started_at: Mapped[datetime | None] = mapped_column(nullable=True)
    completed_at: Mapped[datetime | None] = mapped_column(nullable=True)

    # Celery task id for tracking
    task_id: Mapped[str | None] = mapped_column(String(255))

    # For tournament backtests
    tournament_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("tournaments.id", ondelete="SET NULL"), nullable=True
    )
    tournament_entry_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("tournament_entries.id", ondelete="SET NULL"), nullable=True
    )

    __table_args__ = (
        Index("ix_backtest_status_priority", "status", "priority", "created_at"),
        Index("ix_backtest_user_status", "user_id", "status"),
    )


class CandleData(Base):
    __tablename__ = "candle_data"

    symbol: Mapped[str] = mapped_column(String(20), nullable=False)
    timeframe: Mapped[str] = mapped_column(String(10), nullable=False)
    timestamp: Mapped[datetime] = mapped_column(nullable=False)
    open: Mapped[float] = mapped_column(Numeric(20, 8), nullable=False)
    high: Mapped[float] = mapped_column(Numeric(20, 8), nullable=False)
    low: Mapped[float] = mapped_column(Numeric(20, 8), nullable=False)
    close: Mapped[float] = mapped_column(Numeric(20, 8), nullable=False)
    volume: Mapped[float] = mapped_column(Numeric(20, 8), nullable=False)

    __table_args__ = (
        Index("ix_candle_symbol_tf_time", "symbol", "timeframe", "timestamp", unique=True),
    )
