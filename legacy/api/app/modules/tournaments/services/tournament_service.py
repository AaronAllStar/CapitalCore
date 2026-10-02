import uuid
from datetime import datetime, timezone
from sqlalchemy import select, func
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload
from app.core.exceptions import NotFoundError, ForbiddenError, ConflictError, BusinessRuleError
from app.modules.users.models.user import User
from app.modules.strategies.models.strategy import Strategy
from app.modules.tournaments.models.tournament import (
    Tournament, TournamentEntry, TournamentType, TournamentStatus, EntryStatus,
)
from app.modules.tournaments.schemas.tournament_schema import (
    CreateTournamentRequest, JoinTournamentRequest, LeaderboardEntry,
)
from app.modules.tournaments.engines.scoring import calculate_scores


class TournamentService:
    def __init__(self, db: AsyncSession):
        self.db = db

    async def create(self, user: User, data: CreateTournamentRequest) -> Tournament:
        t = Tournament(
            name=data.name,
            description=data.description,
            type=TournamentType(data.type),
            status=TournamentStatus.DRAFT,
            symbol=data.symbol,
            timeframe=data.timeframe,
            data_start=data.starts_at or data.data_start,
            data_end=data.ends_at or data.data_end,
            max_participants=data.max_participants,
            entry_fee=data.entry_fee,
            prize_pool=data.prize_pool,
            prize_distribution=data.prize_distribution or {1: 60, 2: 30, 3: 10},
            scoring_config=data.scoring_config.model_dump(),
            registration_ends=data.registration_ends,
            starts_at=data.starts_at,
        )
        self.db.add(t)
        await self.db.flush()
        return t

    async def get(self, tournament_id: uuid.UUID) -> Tournament:
        result = await self.db.execute(
            select(Tournament)
            .options(selectinload(Tournament.entries))
            .where(Tournament.id == tournament_id)
        )
        t = result.scalar_one_or_none()
        if not t:
            raise NotFoundError("Tournament", str(tournament_id))
        return t

    async def list_all(self, status: str | None = None, limit: int = 50) -> list[Tournament]:
        query = select(Tournament).options(selectinload(Tournament.entries)).order_by(Tournament.created_at.desc())
        if status:
            query = query.where(Tournament.status == status)
        result = await self.db.execute(query.limit(limit))
        return list(result.scalars().all())

    async def open_registration(self, tournament_id: uuid.UUID) -> Tournament:
        t = await self.get(tournament_id)
        if t.status != TournamentStatus.DRAFT:
            raise BusinessRuleError("Tournament must be in draft to open registration")
        t.status = TournamentStatus.REGISTRATION
        await self.db.flush()
        return t

    async def join(self, tournament_id: uuid.UUID, user: User, data: JoinTournamentRequest) -> TournamentEntry:
        t = await self.get(tournament_id)
        if t.status != TournamentStatus.REGISTRATION:
            raise BusinessRuleError("Registration is closed")
        if t.max_participants and t.participant_count >= t.max_participants:
            raise BusinessRuleError("Tournament is full")

        # Verify strategy
        strategy = await self.db.get(Strategy, data.strategy_id)
        if not strategy:
            raise NotFoundError("Strategy", str(data.strategy_id))
        if strategy.user_id != user.id:
            raise ForbiddenError("Not your strategy")

        # Check duplicate
        existing = await self.db.execute(
            select(TournamentEntry).where(
                TournamentEntry.tournament_id == tournament_id,
                TournamentEntry.user_id == user.id,
            )
        )
        if existing.scalar_one_or_none():
            raise ConflictError("Already joined this tournament")

        entry = TournamentEntry(
            tournament_id=tournament_id,
            user_id=user.id,
            strategy_id=data.strategy_id,
            strategy_snapshot=strategy.rules,
            status=EntryStatus.REGISTERED,
        )
        self.db.add(entry)
        await self.db.flush()
        return entry

    async def start(self, tournament_id: uuid.UUID) -> Tournament:
        """Start tournament and enqueue backtests for all entries."""
        t = await self.get(tournament_id)
        if t.status != TournamentStatus.REGISTRATION:
            raise BusinessRuleError("Must be in registration to start")
        if t.participant_count < (t.min_participants or 2):
            raise BusinessRuleError(f"Need at least {t.min_participants or 2} participants")

        t.status = TournamentStatus.RUNNING
        t.starts_at = datetime.now(timezone.utc)

        # Enqueue backtests
        for entry in t.entries:
            entry.status = EntryStatus.RUNNING
            try:
                from app.core.queue.tasks import backtest_tasks
                task = backtest_tasks.run_backtest_task.delay(
                    str(entry.id),
                    tournament_mode=True,
                    tournament_id=str(tournament_id),
                    entry_id=str(entry.id),
                    strategy_snapshot=entry.strategy_snapshot,
                    symbol=t.symbol,
                    timeframe=t.timeframe,
                    start_date=t.data_start.isoformat(),
                    end_date=t.data_end.isoformat(),
                )
            except Exception:
                pass

        await self.db.flush()
        return t

    async def get_leaderboard(self, tournament_id: uuid.UUID) -> list[LeaderboardEntry]:
        result = await self.db.execute(
            select(TournamentEntry)
            .where(TournamentEntry.tournament_id == tournament_id)
            .order_by(TournamentEntry.rank.asc().nulls_last(), TournamentEntry.score.desc().nulls_last())
        )
        entries = list(result.scalars().all())

        leaderboard = []
        for entry in entries:
            user = await self.db.get(User, entry.user_id)
            leaderboard.append(LeaderboardEntry(
                rank=entry.rank or 0,
                user_id=entry.user_id,
                username=user.username if user else "unknown",
                score=float(entry.score) if entry.score else None,
                pnl_pct=float(entry.pnl_pct) if entry.pnl_pct else None,
                sharpe_ratio=float(entry.sharpe_ratio) if entry.sharpe_ratio else None,
                win_rate=float(entry.win_rate) if entry.win_rate else None,
            ))
        return leaderboard

    async def finalize(self, tournament_id: uuid.UUID) -> Tournament:
        """Score all entries and determine winners."""
        t = await self.get(tournament_id)
        if t.status != TournamentStatus.RUNNING:
            raise BusinessRuleError("Tournament must be running to finalize")

        # Collect completed entry results
        results = []
        for entry in t.entries:
            if entry.status == EntryStatus.COMPLETED and entry.results:
                results.append({
                    "entry_id": str(entry.id),
                    "pnl_pct": float(entry.pnl_pct or 0),
                    "sharpe_ratio": float(entry.sharpe_ratio or 0),
                    "max_drawdown_pct": float(entry.max_drawdown_pct or 100),
                    "win_rate": float(entry.win_rate or 0),
                    "profit_factor": entry.results.get("profit_factor", 0),
                })

        scored = calculate_scores(results, t.scoring_config)
        score_map = {s["entry_id"]: s for s in scored}

        for entry in t.entries:
            if str(entry.id) in score_map:
                s = score_map[str(entry.id)]
                entry.score = s["score"]
                entry.rank = s["rank"]

        t.status = TournamentStatus.COMPLETED
        t.ends_at = datetime.now(timezone.utc)
        await self.db.flush()
        return t
