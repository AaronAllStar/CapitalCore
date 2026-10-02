import uuid
import enum
from datetime import datetime
from sqlalchemy import String, Integer, Text, ForeignKey, Enum as SAEnum, Index
from sqlalchemy import Numeric
from sqlalchemy.dialects.postgresql import UUID, JSONB
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class TournamentType(str, enum.Enum):
    ARENA = "arena"
    DUEL = "duel"
    LEAGUE = "league"
    SPONSORED = "sponsored"


class TournamentStatus(str, enum.Enum):
    DRAFT = "draft"
    REGISTRATION = "registration"
    RUNNING = "running"
    SCORING = "scoring"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class EntryStatus(str, enum.Enum):
    REGISTERED = "registered"
    RUNNING = "running"
    COMPLETED = "completed"
    DISQUALIFIED = "disqualified"


class Tournament(Base):
    __tablename__ = "tournaments"

    name: Mapped[str] = mapped_column(String(200), nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    type: Mapped[str] = mapped_column(
        SAEnum(TournamentType, native_enum=False), nullable=False
    )
    status: Mapped[str] = mapped_column(
        SAEnum(TournamentStatus, native_enum=False), default=TournamentStatus.DRAFT, nullable=False
    )

    # Competition config
    symbol: Mapped[str] = mapped_column(String(20), nullable=False)
    timeframe: Mapped[str] = mapped_column(String(10), nullable=False)
    data_start: Mapped[datetime] = mapped_column(nullable=False)
    data_end: Mapped[datetime] = mapped_column(nullable=False)
    scoring_config: Mapped[dict] = mapped_column(JSONB, nullable=False, default=lambda: {
        "pnl_weight": 0.35, "sharpe_weight": 0.25, "drawdown_weight": 0.20,
        "winrate_weight": 0.10, "pf_weight": 0.10,
    })

    # Limits
    max_participants: Mapped[int | None] = mapped_column(Integer)
    min_participants: Mapped[int] = mapped_column(Integer, default=2)

    # Prizes
    entry_fee: Mapped[float] = mapped_column(Numeric(10, 2), default=0)
    prize_pool: Mapped[float] = mapped_column(Numeric(10, 2), default=0)
    prize_distribution: Mapped[dict | None] = mapped_column(JSONB)  # {1: 50, 2: 30, 3: 20}

    # Sponsor
    sponsor_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id")
    )

    # Dates
    registration_starts: Mapped[datetime | None] = mapped_column(nullable=True)
    registration_ends: Mapped[datetime | None] = mapped_column(nullable=True)
    starts_at: Mapped[datetime | None] = mapped_column(nullable=True)
    ends_at: Mapped[datetime | None] = mapped_column(nullable=True)

    # Relationships
    entries = relationship("TournamentEntry", back_populates="tournament", lazy="selectin", cascade="all, delete-orphan")

    @property
    def participant_count(self) -> int:
        return len(self.entries)

    __table_args__ = (
        Index("ix_tournament_status_type", "status", "type"),
    )


class TournamentEntry(Base):
    __tablename__ = "tournament_entries"

    tournament_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("tournaments.id", ondelete="CASCADE"), nullable=False
    )
    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    strategy_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("strategies.id"), nullable=False
    )
    strategy_snapshot: Mapped[dict] = mapped_column(JSONB, nullable=False)

    status: Mapped[str] = mapped_column(
        SAEnum(EntryStatus, native_enum=False), default=EntryStatus.REGISTERED, nullable=False
    )

    # Results
    score: Mapped[float | None] = mapped_column(Numeric(10, 4))
    rank: Mapped[int | None] = mapped_column(Integer)
    pnl_pct: Mapped[float | None] = mapped_column(Numeric(10, 4))
    sharpe_ratio: Mapped[float | None] = mapped_column(Numeric(10, 4))
    max_drawdown_pct: Mapped[float | None] = mapped_column(Numeric(10, 4))
    win_rate: Mapped[float | None] = mapped_column(Numeric(10, 4))
    total_trades: Mapped[int | None] = mapped_column(Integer)
    results: Mapped[dict | None] = mapped_column(JSONB)

    # Backtest reference
    backtest_job_id: Mapped[uuid.UUID | None] = mapped_column(
        UUID(as_uuid=True), ForeignKey("backtest_jobs.id", ondelete="SET NULL")
    )

    tournament = relationship("Tournament", back_populates="entries")

    __table_args__ = (
        Index("ix_tournament_entry_unique", "tournament_id", "user_id", unique=True),
        Index("ix_tournament_entry_rank", "tournament_id", "rank"),
    )
