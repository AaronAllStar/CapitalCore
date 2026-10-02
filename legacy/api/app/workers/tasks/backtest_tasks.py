"""Backtest Celery tasks — run in worker processes."""
import asyncio
from datetime import datetime
from celery import shared_task
from app.core.logger import get_logger

logger = get_logger("workers.backtest")


@shared_task(bind=True, max_retries=2, default_retry_delay=30, acks_late=True)
def run_backtest_task(
    self,
    backtest_id: str,
    tournament_mode: bool = False,
    tournament_id: str | None = None,
    entry_id: str | None = None,
    strategy_snapshot: dict | None = None,
    symbol: str | None = None,
    timeframe: str | None = None,
    start_date: str | None = None,
    end_date: str | None = None,
):
    """Execute a backtest job."""
    logger.info("backtest_started", backtest_id=backtest_id, tournament=tournament_mode)

    async def _run():
        from app.core.db.session import async_session_factory
        from app.modules.backtest.models.backtest_job import BacktestJob, BacktestStatus
        from app.modules.strategies.models.strategy import Strategy
        from app.modules.backtest.models.backtest_job import CandleData
        from app.modules.backtest.engines.runner import run_backtest
        from sqlalchemy import select
        import pandas as pd

        async with async_session_factory() as db:
            try:
                job = await db.get(BacktestJob, backtest_id)
                if not job:
                    logger.error("backtest_job_not_found", backtest_id=backtest_id)
                    return

                job.status = BacktestStatus.RUNNING
                job.started_at = datetime.utcnow()
                job.task_id = self.request.id
                await db.commit()

                # Get strategy rules
                if tournament_mode and strategy_snapshot:
                    rules = strategy_snapshot
                else:
                    strategy = await db.get(Strategy, job.strategy_id)
                    if not strategy:
                        raise ValueError("Strategy not found")
                    rules = strategy.rules

                # Load candle data
                query = select(CandleData).where(
                    CandleData.symbol == (symbol or job.symbol),
                    CandleData.timeframe == (timeframe or job.timeframe),
                    CandleData.timestamp >= datetime.fromisoformat(start_date) if start_date else job.start_date,
                    CandleData.timestamp <= datetime.fromisoformat(end_date) if end_date else job.end_date,
                ).order_by(CandleData.timestamp)

                result = await db.execute(query)
                candles = result.scalars().all()

                if len(candles) < 50:
                    raise ValueError(f"Insufficient data: {len(candles)} candles (need 50+)")

                df = pd.DataFrame([{
                    "timestamp": c.timestamp,
                    "open": float(c.open),
                    "high": float(c.high),
                    "low": float(c.low),
                    "close": float(c.close),
                    "volume": float(c.volume),
                } for c in candles])

                # Run backtest
                output = run_backtest(
                    df=df,
                    rules=rules,
                    initial_capital=float(job.initial_capital),
                )

                # Save results
                job.status = BacktestStatus.COMPLETED
                job.results = output["results"]
                job.equity_curve = output["equity_curve"]
                job.trades = output["trades"]
                job.execution_time_ms = output["execution_time_ms"]
                job.candle_count = output["candle_count"]
                job.completed_at = datetime.utcnow()

                # Update strategy stats if not tournament
                if not tournament_mode:
                    strategy = await db.get(Strategy, job.strategy_id)
                    if strategy:
                        strategy.total_backtests += 1
                        m = output["results"]
                        if m["pnl_pct"] > 0:
                            strategy.total_wins += 1
                        if strategy.best_sharpe is None or m["sharpe_ratio"] > strategy.best_sharpe:
                            strategy.best_sharpe = m["sharpe_ratio"]
                        if strategy.best_pnl_pct is None or m["pnl_pct"] > strategy.best_pnl_pct:
                            strategy.best_pnl_pct = m["pnl_pct"]
                        wr = (strategy.total_wins / strategy.total_backtests * 100) if strategy.total_backtests > 0 else 0
                        strategy.win_rate = wr

                # If tournament mode, update entry
                if tournament_mode and entry_id:
                    from app.modules.tournaments.models.tournament import TournamentEntry, EntryStatus
                    entry = await db.get(TournamentEntry, entry_id)
                    if entry:
                        entry.status = EntryStatus.COMPLETED
                        entry.backtest_job_id = job.id
                        m = output["results"]
                        entry.results = m
                        entry.pnl_pct = m["pnl_pct"]
                        entry.sharpe_ratio = m["sharpe_ratio"]
                        entry.max_drawdown_pct = m["max_drawdown_pct"]
                        entry.win_rate = m["win_rate"]
                        entry.total_trades = m["total_trades"]

                await db.commit()
                logger.info("backtest_completed", backtest_id=backtest_id, trades=output["results"]["total_trades"])

            except Exception as e:
                logger.error("backtest_failed", backtest_id=backtest_id, error=str(e))
                try:
                    job = await db.get(BacktestJob, backtest_id)
                    if job:
                        job.status = BacktestStatus.FAILED
                        job.error_message = str(e)
                        job.completed_at = datetime.utcnow()
                        await db.commit()
                except Exception:
                    pass
                raise

    asyncio.run(_run())
