from fastapi import APIRouter, Depends, Query
from sqlalchemy import select, desc
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.core.cache.redis import get_cache, CacheService
from app.modules.users.models.user import User

router = APIRouter()


@router.get("/global")
async def global_leaderboard(
    limit: int = Query(50, le=100),
    offset: int = Query(0, ge=0),
    db: AsyncSession = Depends(get_session),
    cache: CacheService = Depends(get_cache),
):
    cache_key = f"leaderboard:global:{limit}:{offset}"
    cached = await cache.get(cache_key)
    if cached:
        return cached

    result = await db.execute(
        select(User)
        .where(User.is_active == True, User.is_banned == False)  # noqa: E712
        .where(User.total_wins + User.total_losses > 0)
        .order_by(desc(User.rating))
        .limit(limit)
        .offset(offset)
    )
    users = list(result.scalars().all())

    data = [
        {
            "rank": offset + i + 1,
            "user_id": str(u.id),
            "username": u.username,
            "display_name": u.display_name,
            "rating": u.rating,
            "peak_rating": u.peak_rating,
            "wins": u.total_wins,
            "losses": u.total_losses,
            "tournaments": u.total_tournaments,
        }
        for i, u in enumerate(users)
    ]

    await cache.set(cache_key, data, ttl=60)
    return data


@router.get("/user/{user_id}")
async def user_stats(
    user_id: str,
    db: AsyncSession = Depends(get_session),
):
    user = await db.get(User, user_id)
    if not user:
        return {"error": "User not found"}

    # Get rank
    rank_result = await db.execute(
        select(User)
        .where(User.rating > user.rating, User.is_active == True)  # noqa: E712
    )
    rank = len(list(rank_result.scalars().all())) + 1

    return {
        "user_id": str(user.id),
        "username": user.username,
        "rating": user.rating,
        "peak_rating": user.peak_rating,
        "rank": rank,
        "wins": user.total_wins,
        "losses": user.total_losses,
        "tournaments": user.total_tournaments,
        "backtests": user.total_backtests,
    }
