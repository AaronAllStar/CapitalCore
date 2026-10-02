import uuid
from datetime import datetime
from pydantic import BaseModel, Field


class ScoringConfig(BaseModel):
    pnl_weight: float = 0.35
    sharpe_weight: float = 0.25
    drawdown_weight: float = 0.20
    winrate_weight: float = 0.10
    pf_weight: float = 0.10


class CreateTournamentRequest(BaseModel):
    name: str = Field(min_length=1, max_length=200)
    description: str | None = None
    type: str = "arena"
    symbol: str = Field(min_length=1, max_length=20)
    timeframe: str = Field(min_length=1, max_length=10)
    data_start: datetime
    data_end: datetime
    max_participants: int | None = Field(None, ge=2, le=1000)
    entry_fee: float = Field(ge=0, default=0)
    prize_pool: float = Field(ge=0, default=0)
    prize_distribution: dict | None = None
    scoring_config: ScoringConfig = ScoringConfig()
    registration_ends: datetime | None = None
    starts_at: datetime | None = None


class TournamentEntryResponse(BaseModel):
    id: uuid.UUID
    tournament_id: uuid.UUID
    user_id: uuid.UUID
    strategy_id: uuid.UUID
    status: str
    score: float | None
    rank: int | None
    pnl_pct: float | None
    sharpe_ratio: float | None
    max_drawdown_pct: float | None
    win_rate: float | None
    total_trades: int | None
    created_at: datetime

    model_config = {"from_attributes": True}


class TournamentResponse(BaseModel):
    id: uuid.UUID
    name: str
    description: str | None
    type: str
    status: str
    symbol: str
    timeframe: str
    max_participants: int | None
    entry_fee: float
    prize_pool: float
    registration_ends: datetime | None
    starts_at: datetime | None
    ends_at: datetime | None
    participant_count: int
    created_at: datetime

    model_config = {"from_attributes": True}


class JoinTournamentRequest(BaseModel):
    strategy_id: uuid.UUID


class LeaderboardEntry(BaseModel):
    rank: int
    user_id: uuid.UUID
    username: str
    score: float | None
    pnl_pct: float | None
    sharpe_ratio: float | None
    win_rate: float | None
