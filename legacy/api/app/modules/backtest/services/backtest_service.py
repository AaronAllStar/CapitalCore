import uuid
from datetime import datetime, timezone
from sqlalchemy import select, func, and_
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.exceptions import NotFoundError, ForbiddenError, PlanLimitError, BusinessRuleError
from app.modules.users.models.user import User
from app.modules.strategies.models.strategy import Strategy
from app.modules.backtest.models.backtest_job import BacktestJob, BacktestStatus, BacktestPriority
from app.modules.backtest.schemas.backtest_schema import CreateBacktestRequest

PLAN_LIMITS = {
    "free": {"per_month": 20, "max_candles": 50_000},
    "basic": {"per_month": 200, "max_candles": 200_000},
    "premium": {"per_month": 9999, "max_candles": 500_000},
}


class BacktestService:
    def __init__(self, db: AsyncSession):
        self.db = db

    async def create(self, user: User, data: CreateBacktestRequest) -> BacktestJob:
        # Verify strategy exists and user has access
        strategy = await self.db.get(Strategy, data.strategy_id)
        if not strategy:
            raise NotFoundError("Strategy", str(data.strategy_id))
        if strategy.user_id != user.id and not strategy.is_public:
            raise ForbiddenError("Cannot backtest this strategy")

        # Check limits
        limits = PLAN_LIMITS.get(user.plan, PLAN_LIMITS["free"])
        now = datetime.now(timezone.utc)
        month_start = now.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
        count = await self.db.execute(
            select(func.count()).where(
                and_(
                    BacktestJob.user_id == user.id,
                    BacktestJob.created_at >= month_start,
                    BacktestJob.tournament_id.is_(None),
                )
            )
        )
        if count.scalar() >= limits["per_month"]:
            raise PlanLimitError(f"Monthly backtest limit ({limits['per_month']})", plan=user.plan)

        job = BacktestJob(
            user_id=user.id,
            strategy_id=data.strategy_id,
            strategy_version=strategy.version,
            strategy_snapshot=strategy.rules,
            symbol=data.symbol,
            timeframe=data.timeframe,
            start_date=data.start_date,
            end_date=data.end_date,
            initial_capital=data.initial_capital,
            status=BacktestStatus.QUEUED,
            priority=BacktestPriority.NORMAL,
        )
        self.db.add(job)
        await self.db.flush()

        # Enqueue async task
        try:
            from app.core.queue.tasks import backtest_tasks
            task = backtest_tasks.run_backtest_task.delay(str(job.id))
            job.task_id = task.id
            await self.db.flush()
        except Exception:
            pass  # Job stays queued, can be retried

        return job

    async def get(self, job_id: uuid.UUID, user_id: uuid.UUID) -> BacktestJob:
        job = await self.db.get(BacktestJob, job_id)
        if not job:
            raise NotFoundError("Backtest", str(job_id))
        if job.user_id != user_id and job.tournament_id is None:
            raise ForbiddenError("Not your backtest")
        return job

    async def list_user(self, user_id: uuid.UUID, limit: int = 50) -> list[BacktestJob]:
        result = await self.db.execute(
            select(BacktestJob)
            .where(BacktestJob.user_id == user_id)
            .order_by(BacktestJob.created_at.desc())
            .limit(limit)
        )
        return list(result.scalars().all())

    async def cancel(self, job_id: uuid.UUID, user_id: uuid.UUID) -> BacktestJob:
        job = await self.get(job_id, user_id)
        if job.status not in (BacktestStatus.QUEUED, BacktestStatus.RUNNING):
            raise BusinessRuleError("Can only cancel queued or running backtests")
        job.status = BacktestStatus.CANCELLED
        if job.task_id:
            try:
                from app.core.queue.celery import celery_app
                celery_app.control.revoke(job.task_id, terminate=True)
            except Exception:
                pass
        await self.db.flush()
        return job

    async def get_trades(self, job_id: uuid.UUID, user_id: uuid.UUID) -> list:
        job = await self.get(job_id, user_id)
        return job.trades or []

    async def get_equity(self, job_id: uuid.UUID, user_id: uuid.UUID) -> list:
        job = await self.get(job_id, user_id)
        return job.equity_curve or []
