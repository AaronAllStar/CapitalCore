"""Tournament Celery tasks."""
import asyncio
from celery import shared_task
from app.core.logger import get_logger

logger = get_logger("workers.tournament")


@shared_task(bind=True)
def finalize_tournament_task(self, tournament_id: str):
    """Auto-finalize a tournament when it ends."""
    async def _run():
        from app.core.db.session import async_session_factory
        from app.modules.tournaments.services.tournament_service import TournamentService

        async with async_session_factory() as db:
            svc = TournamentService(db)
            try:
                t = await svc.finalize(tournament_id)
                logger.info("tournament_finalized", tournament_id=tournament_id)
            except Exception as e:
                logger.error("tournament_finalize_failed", tournament_id=tournament_id, error=str(e))
                raise

    asyncio.run(_run())
