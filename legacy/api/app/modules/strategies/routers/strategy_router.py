import uuid
from fastapi import APIRouter, Depends, Query
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import get_current_user, get_optional_user
from app.modules.users.models.user import User
from app.modules.strategies.schemas.strategy_schema import (
    CreateStrategyRequest,
    CreateFromTemplateRequest,
    CreateFromAIRequest,
    UpdateStrategyRequest,
    StrategyResponse,
    StrategyListItem,
    ValidateRulesRequest,
    ValidateResponse,
    TemplateListItem,
    TemplateResponse,
    IndicatorSpecResponse,
    CritiqueRequest,
    ExplainRequest,
)
from app.modules.strategies.services.strategy_service import StrategyService
from app.modules.strategies.services.templates import get_template_list, get_template
from app.modules.strategies.services.indicator_registry import get_registry_json

router = APIRouter()


# ── Create ──────────────────────────────────────────────────

@router.post("", response_model=StrategyResponse, status_code=201)
async def create_strategy(
    data: CreateStrategyRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Create a strategy manually with full rules."""
    svc = StrategyService(db)
    return await svc.create(user, data)


@router.post("/from-template", response_model=StrategyResponse, status_code=201)
async def create_from_template(
    data: CreateFromTemplateRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Create a strategy from a built-in template."""
    svc = StrategyService(db)
    return await svc.create_from_template(user, data)


@router.post("/from-ai")
async def create_from_ai(
    data: CreateFromAIRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """
    Convert natural language to structured strategy rules.
    If save=True, also persists the strategy.
    """
    svc = StrategyService(db)
    return await svc.create_from_ai(user, data)


# ── Templates ───────────────────────────────────────────────

@router.get("/templates", response_model=list[TemplateListItem])
async def list_templates():
    """List all built-in strategy templates."""
    return get_template_list()


@router.get("/templates/{template_id}", response_model=TemplateResponse)
async def get_template_detail(template_id: str):
    """Get a specific template with its full rules."""
    from app.core.exceptions import NotFoundError
    template = get_template(template_id)
    if not template:
        raise NotFoundError("Template", template_id)
    return template


# ── Indicators ──────────────────────────────────────────────

@router.get("/indicators", response_model=list[IndicatorSpecResponse])
async def list_indicators():
    """List all available indicators with params. Use for frontend dropdowns."""
    return get_registry_json()


# ── AI features ─────────────────────────────────────────────

@router.post("/ai/critique")
async def ai_critique(
    data: CritiqueRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Get AI analysis and suggestions for a strategy."""
    svc = StrategyService(db)
    analysis = await svc.ai_critique(data.rules)
    return {"analysis": analysis}


@router.post("/ai/explain")
async def ai_explain(
    data: ExplainRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Get AI explanation of an indicator."""
    svc = StrategyService(db)
    explanation = await svc.ai_explain(data.indicator)
    return {"indicator": data.indicator, "explanation": explanation}


# ── List & Read ─────────────────────────────────────────────

@router.get("", response_model=list[StrategyListItem])
async def list_strategies(
    status: str | None = Query(None),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """List your strategies."""
    svc = StrategyService(db)
    return await svc.list_user(user.id, status=status)


@router.get("/public", response_model=list[StrategyListItem])
async def list_public_strategies(
    limit: int = Query(50, le=100),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_session),
):
    """Browse public strategies."""
    svc = StrategyService(db)
    return await svc.list_public(limit=limit, offset=offset)


@router.get("/{strategy_id}", response_model=StrategyResponse)
async def get_strategy(
    strategy_id: uuid.UUID,
    user: User | None = Depends(get_optional_user),
    db: AsyncSession = Depends(get_session),
):
    """Get a strategy by ID."""
    svc = StrategyService(db)
    return await svc.get(strategy_id, user_id=user.id if user else None)


# ── Update & Delete ─────────────────────────────────────────

@router.patch("/{strategy_id}", response_model=StrategyResponse)
async def update_strategy(
    strategy_id: uuid.UUID,
    data: UpdateStrategyRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Update a strategy. Rules change creates a new version."""
    svc = StrategyService(db)
    strategy = await svc.get(strategy_id, user_id=user.id)
    return await svc.update(strategy, user, data)


@router.delete("/{strategy_id}", status_code=204)
async def delete_strategy(
    strategy_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Archive a strategy (soft delete)."""
    svc = StrategyService(db)
    strategy = await svc.get(strategy_id, user_id=user.id)
    await svc.delete(strategy, user)


@router.post("/{strategy_id}/duplicate", response_model=StrategyResponse, status_code=201)
async def duplicate_strategy(
    strategy_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Duplicate a strategy (own or public)."""
    svc = StrategyService(db)
    return await svc.duplicate(strategy_id, user)


@router.post("/{strategy_id}/copy", response_model=StrategyResponse, status_code=201)
async def copy_strategy(
    strategy_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    """Legacy copy endpoint — redirects to duplicate."""
    svc = StrategyService(db)
    return await svc.copy(strategy_id, user)


# ── Validation ──────────────────────────────────────────────

@router.post("/validate", response_model=ValidateResponse)
async def validate_rules_endpoint(
    data: ValidateRulesRequest,
    db: AsyncSession = Depends(get_session),
):
    """Validate rules without saving. For real-time builder feedback."""
    svc = StrategyService(db)
    return await svc.validate(data.rules)
