import uuid
from datetime import datetime
from pydantic import BaseModel, Field


# ── Rule Engine Schema ─────────────────────────────────────

class ConditionSchema(BaseModel):
    type: str = "condition"
    indicator: str
    params: dict[str, float] = {}
    operator: str
    target: dict = {}


class RiskManagementSchema(BaseModel):
    stop_loss_pct: float = Field(ge=0, le=100, default=2.0)
    take_profit_pct: float = Field(ge=0, le=100, default=4.0)
    position_size_pct: float = Field(gt=0, le=100, default=5.0)
    max_open_positions: int = Field(ge=1, le=50, default=1)


class StrategyRulesSchema(BaseModel):
    version: str = "1.0"
    name: str = ""
    asset: str = "BTC/USDT"
    timeframe: str = "1h"
    entry_rules: list[ConditionSchema] = []
    exit_rules: list[ConditionSchema] = []
    risk_management: RiskManagementSchema = RiskManagementSchema()


# ── Request Schemas ─────────────────────────────────────────

class CreateStrategyRequest(BaseModel):
    """Create strategy manually with full rules."""
    name: str = Field(min_length=1, max_length=200)
    description: str | None = Field(None, max_length=5000)
    rules: StrategyRulesSchema
    tags: list[str] = Field(default=[], max_length=10)
    is_public: bool = False


class CreateFromTemplateRequest(BaseModel):
    """Create strategy from a built-in template, optionally overriding fields."""
    template_id: str = Field(min_length=1, max_length=100)
    name: str | None = Field(None, max_length=200)
    asset: str | None = None
    timeframe: str | None = None
    risk_management: RiskManagementSchema | None = None
    tags: list[str] = Field(default=[], max_length=10)
    is_public: bool = False


class CreateFromAIRequest(BaseModel):
    """Create strategy from natural language description using AI."""
    description: str = Field(min_length=10, max_length=3000)
    tags: list[str] = Field(default=[], max_length=10)
    is_public: bool = False
    save: bool = True  # If False, just returns the rules without saving


class UpdateStrategyRequest(BaseModel):
    name: str | None = Field(None, min_length=1, max_length=200)
    description: str | None = Field(None, max_length=5000)
    rules: StrategyRulesSchema | None = None
    tags: list[str] | None = Field(None, max_length=10)
    is_public: bool | None = None
    changelog: str | None = Field(None, max_length=500)
    status: str | None = None


class ValidateRulesRequest(BaseModel):
    rules: StrategyRulesSchema


class CritiqueRequest(BaseModel):
    rules: StrategyRulesSchema


class ExplainRequest(BaseModel):
    indicator: str


# ── Response Schemas ────────────────────────────────────────

class StrategyVersionResponse(BaseModel):
    id: uuid.UUID
    version: int
    changelog: str | None
    created_at: datetime

    model_config = {"from_attributes": True}


class StrategyResponse(BaseModel):
    id: uuid.UUID
    user_id: uuid.UUID
    name: str
    description: str | None
    version: int
    status: str
    source: str
    template_id: str | None = None
    is_public: bool
    is_for_sale: bool
    price: float | None
    rules: dict
    tags: list[str] | None
    total_backtests: int
    total_wins: int
    best_sharpe: float | None
    best_pnl_pct: float | None
    win_rate: float | None
    created_at: datetime
    updated_at: datetime
    versions: list[StrategyVersionResponse] = []

    model_config = {"from_attributes": True}


class StrategyListItem(BaseModel):
    id: uuid.UUID
    name: str
    version: int
    status: str
    source: str
    is_public: bool
    tags: list[str] | None
    total_backtests: int
    win_rate: float | None
    best_sharpe: float | None
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class ValidateResponse(BaseModel):
    valid: bool
    errors: list[str] = []
    warnings: list[str] = []


class AIConvertResponse(BaseModel):
    rules: StrategyRulesSchema
    explanation: str


class TemplateListItem(BaseModel):
    id: str
    name: str
    description: str
    category: str
    difficulty: str
    recommended_assets: list[str]


class TemplateResponse(BaseModel):
    id: str
    name: str
    description: str
    category: str
    difficulty: str
    recommended_assets: list[str]
    rules: StrategyRulesSchema


class IndicatorSpecResponse(BaseModel):
    name: str
    label: str
    description: str
    category: str
    outputs: int
    output_labels: list[str]
    params: list[dict]
