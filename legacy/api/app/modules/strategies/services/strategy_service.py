import uuid
from datetime import datetime, timezone
from sqlalchemy import select, func, and_
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload
from app.core.exceptions import NotFoundError, ForbiddenError, PlanLimitError, BusinessRuleError
from app.modules.users.models.user import User
from app.modules.strategies.models.strategy import (
    Strategy, StrategyVersion, StrategyCopy, StrategyStatus, StrategySource,
)
from app.modules.strategies.schemas.strategy_schema import (
    CreateStrategyRequest,
    CreateFromTemplateRequest,
    CreateFromAIRequest,
    UpdateStrategyRequest,
    StrategyRulesSchema,
    ValidateResponse,
)
from app.modules.strategies.services.validator import validate_rules
from app.modules.strategies.services.templates import get_template
from app.modules.strategies.services.ai_service import natural_language_to_rules, critique_strategy, explain_indicator

PLAN_LIMITS = {
    "free": {"max_strategies": 5, "copies_per_day": 5},
    "basic": {"max_strategies": 25, "copies_per_day": 50},
    "premium": {"max_strategies": 999, "copies_per_day": 999},
}


class StrategyService:
    def __init__(self, db: AsyncSession):
        self.db = db

    # ── Create manually ─────────────────────────────────────
    async def create(self, user: User, data: CreateStrategyRequest) -> Strategy:
        await self._check_limits(user)

        errors, _ = validate_rules(data.rules)
        if errors:
            from app.core.exceptions import ValidationError
            raise ValidationError(
                detail="Validation failed",
                errors=[{"field": "rules", "message": e} for e in errors],
            )

        strategy = Strategy(
            user_id=user.id,
            name=data.name,
            description=data.description,
            rules=data.rules.model_dump(),
            tags=data.tags,
            is_public=data.is_public,
            source=StrategySource.MANUAL,
        )
        self.db.add(strategy)
        await self.db.flush()
        await self._save_version(strategy, "Initial version")
        return strategy

    # ── Create from template ────────────────────────────────
    async def create_from_template(self, user: User, data: CreateFromTemplateRequest) -> Strategy:
        template = get_template(data.template_id)
        if not template:
            raise NotFoundError("Template", data.template_id)

        await self._check_limits(user)

        # Merge template rules with overrides
        rules_dict = {**template["rules"]}
        if data.asset:
            rules_dict["asset"] = data.asset
        if data.timeframe:
            rules_dict["timeframe"] = data.timeframe
        if data.risk_management:
            rules_dict["risk_management"] = data.risk_management.model_dump()

        name = data.name or template["name"]

        rules = StrategyRulesSchema(**rules_dict)
        errors, _ = validate_rules(rules)
        if errors:
            from app.core.exceptions import ValidationError
            raise ValidationError(
                detail="Template validation failed",
                errors=[{"field": "rules", "message": e} for e in errors],
            )

        strategy = Strategy(
            user_id=user.id,
            name=name,
            description=template["description"],
            rules=rules_dict,
            tags=data.tags or [template["category"]],
            is_public=data.is_public,
            source=StrategySource.TEMPLATE,
            template_id=data.template_id,
        )
        self.db.add(strategy)
        await self.db.flush()
        await self._save_version(strategy, f"Created from template: {template['name']}")
        return strategy

    # ── Create from natural language (AI) ───────────────────
    async def create_from_ai(self, user: User, data: CreateFromAIRequest) -> dict:
        result = await natural_language_to_rules(data.description)
        rules = StrategyRulesSchema(**result["rules"])

        if not data.save:
            errors, warnings = validate_rules(rules)
            return {
                "rules": rules.model_dump(),
                "explanation": result.get("explanation", ""),
                "warnings": warnings,
                "valid": len(errors) == 0,
                "errors": errors,
            }

        await self._check_limits(user)

        strategy = Strategy(
            user_id=user.id,
            name=rules.name,
            description=result.get("explanation"),
            rules=rules.model_dump(),
            tags=data.tags,
            is_public=data.is_public,
            source=StrategySource.AI,
            natural_input=data.description,
        )
        self.db.add(strategy)
        await self.db.flush()
        await self._save_version(strategy, "Created by AI")

        _, warnings = validate_rules(rules)
        return {
            "strategy_id": str(strategy.id),
            "rules": rules.model_dump(),
            "explanation": result.get("explanation", ""),
            "warnings": warnings,
        }

    # ── CRUD ────────────────────────────────────────────────
    async def get(self, strategy_id: uuid.UUID, user_id: uuid.UUID | None = None) -> Strategy:
        result = await self.db.execute(
            select(Strategy)
            .options(selectinload(Strategy.versions))
            .where(Strategy.id == strategy_id)
        )
        strategy = result.scalar_one_or_none()
        if not strategy:
            raise NotFoundError("Strategy", str(strategy_id))
        if not strategy.is_public and user_id and strategy.user_id != user_id:
            raise ForbiddenError("This strategy is private")
        return strategy

    async def list_user(self, user_id: uuid.UUID, status: str | None = None) -> list[Strategy]:
        query = select(Strategy).where(Strategy.user_id == user_id).order_by(Strategy.updated_at.desc())
        if status:
            query = query.where(Strategy.status == status)
        result = await self.db.execute(query)
        return list(result.scalars().all())

    async def list_public(self, limit: int = 50, offset: int = 0) -> list[Strategy]:
        result = await self.db.execute(
            select(Strategy)
            .where(Strategy.is_public == True, Strategy.status == StrategyStatus.ACTIVE)  # noqa: E712
            .order_by(Strategy.best_sharpe.desc().nulls_last())
            .limit(limit)
            .offset(offset)
        )
        return list(result.scalars().all())

    async def update(self, strategy: Strategy, user: User, data: UpdateStrategyRequest) -> Strategy:
        if strategy.user_id != user.id:
            raise ForbiddenError("Not your strategy")

        if data.rules is not None:
            errors, _ = validate_rules(data.rules)
            if errors:
                from app.core.exceptions import ValidationError
                raise ValidationError(
                    detail="Validation failed",
                    errors=[{"field": "rules", "message": e} for e in errors],
                )
            strategy.version += 1
            strategy.rules = data.rules.model_dump()
            await self._save_version(strategy, data.changelog or f"Version {strategy.version}")

        if data.name is not None:
            strategy.name = data.name
        if data.description is not None:
            strategy.description = data.description
        if data.tags is not None:
            strategy.tags = data.tags
        if data.is_public is not None:
            strategy.is_public = data.is_public
        if data.status is not None:
            strategy.status = StrategyStatus(data.status)

        await self.db.flush()
        await self.db.refresh(strategy, attribute_names=["versions"])
        return strategy

    async def delete(self, strategy: Strategy, user: User) -> None:
        if strategy.user_id != user.id:
            raise ForbiddenError("Not your strategy")
        strategy.status = StrategyStatus.ARCHIVED
        await self.db.flush()

    # ── Duplicate (replaces copy) ───────────────────────────
    async def duplicate(self, strategy_id: uuid.UUID, user: User) -> Strategy:
        original = await self.get(strategy_id, user_id=user.id)
        if not original.is_public and original.user_id != user.id:
            raise ForbiddenError("Strategy is private")

        await self._check_limits(user)

        copy = Strategy(
            user_id=user.id,
            name=f"{original.name} (copy)",
            description=original.description,
            rules=original.rules,
            tags=original.tags,
            is_public=False,
            source=original.source,
            template_id=original.template_id,
        )
        self.db.add(copy)

        if original.user_id != user.id:
            self.db.add(StrategyCopy(original_id=original.id, copied_by=user.id))

        await self.db.flush()
        await self._save_version(copy, "Duplicated")
        return copy

    # ── Legacy copy (for old route) ─────────────────────────
    async def copy(self, strategy_id: uuid.UUID, user: User) -> Strategy:
        return await self.duplicate(strategy_id, user)

    # ── Validate ────────────────────────────────────────────
    async def validate(self, rules: StrategyRulesSchema) -> ValidateResponse:
        errors, warnings = validate_rules(rules)
        return ValidateResponse(valid=len(errors) == 0, errors=errors, warnings=warnings)

    # ── AI features ─────────────────────────────────────────
    async def ai_critique(self, rules: StrategyRulesSchema) -> str:
        return await critique_strategy(rules.model_dump())

    async def ai_explain(self, indicator_name: str) -> str:
        return await explain_indicator(indicator_name)

    # ── Private helpers ─────────────────────────────────────
    async def _check_limits(self, user: User) -> None:
        plan = user.plan
        limits = PLAN_LIMITS.get(plan, PLAN_LIMITS["free"])
        count = await self.db.execute(
            select(func.count()).where(
                Strategy.user_id == user.id,
                Strategy.status != StrategyStatus.ARCHIVED,
            )
        )
        if count.scalar() >= limits["max_strategies"]:
            raise PlanLimitError(
                f"{plan} plan allows {limits['max_strategies']} strategies. Upgrade to create more.",
                plan=plan,
            )

    async def _save_version(self, strategy: Strategy, changelog: str) -> None:
        self.db.add(StrategyVersion(
            strategy_id=strategy.id,
            version=strategy.version,
            rules=strategy.rules,
            changelog=changelog,
        ))
        await self.db.flush()
