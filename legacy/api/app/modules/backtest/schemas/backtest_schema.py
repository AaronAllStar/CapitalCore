import uuid
from datetime import datetime
from pydantic import BaseModel, Field


class CreateBacktestRequest(BaseModel):
    strategy_id: uuid.UUID
    symbol: str = Field(min_length=1, max_length=20)
    timeframe: str = Field(min_length=1, max_length=10)
    start_date: datetime
    end_date: datetime
    initial_capital: float = Field(gt=0, default=10000)


class BacktestResultMetrics(BaseModel):
    pnl_pct: float
    total_trades: int
    win_rate: float
    sharpe_ratio: float
    max_drawdown_pct: float
    profit_factor: float
    avg_win: float
    avg_loss: float
    best_trade: float
    worst_trade: float
    avg_trade_duration_hours: float


class BacktestResponse(BaseModel):
    id: uuid.UUID
    user_id: uuid.UUID
    strategy_id: uuid.UUID
    strategy_version: int
    status: str
    priority: str
    symbol: str
    timeframe: str
    start_date: datetime
    end_date: datetime
    initial_capital: float
    results: BacktestResultMetrics | None = None
    equity_curve: list | None = None
    trades: list | None = None
    error_message: str | None = None
    execution_time_ms: int | None = None
    candle_count: int | None = None
    created_at: datetime
    started_at: datetime | None = None
    completed_at: datetime | None = None

    model_config = {"from_attributes": True}


class BacktestListItem(BaseModel):
    id: uuid.UUID
    strategy_id: uuid.UUID
    status: str
    symbol: str
    timeframe: str
    results: BacktestResultMetrics | None = None
    error_message: str | None = None
    execution_time_ms: int | None = None
    created_at: datetime
    completed_at: datetime | None = None

    model_config = {"from_attributes": True}
