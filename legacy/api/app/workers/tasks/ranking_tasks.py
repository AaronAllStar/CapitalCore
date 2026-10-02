"""Ranking Celery tasks."""
import asyncio
from celery import shared_task
from app.core.logger import get_logger

logger = get_logger("workers.ranking")


@shared_task(bind=True)
def recalculate_ratings_task(self):
    """Periodic task to recalculate global ratings."""
    async def _run():
        from app.core.db.session import async_session_factory
        from sqlalchemy import select, desc
        from app.modules.users.models.user import User

        async with async_session_factory() as db:
            result = await db.execute(
                select(User).order_by(desc(User.rating)).limit(100)
            )
            users = list(result.scalars().all())
            logger.info("ratings_recalculated", top_count=len(users))

    asyncio.run(_run())


@shared_task(bind=True)
def update_leaderboard_cache_task(self):
    """Refresh cached leaderboards."""
    async def _run():
        from app.core.cache.redis import get_redis, CacheService
        r = await get_redis()
        cache = CacheService(r)
        await cache.delete_pattern("leaderboard:*")
        logger.info("leaderboard_cache_cleared")

    asyncio.run(_run())
