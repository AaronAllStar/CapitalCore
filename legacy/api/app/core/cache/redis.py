from __future__ import annotations
import orjson
from typing import Any
import redis.asyncio as redis
from app.core.config import get_settings
from app.core.logger import get_logger

logger = get_logger(__name__)
settings = get_settings()

_pool: redis.ConnectionPool | None = None


def get_redis_pool() -> redis.ConnectionPool:
    global _pool
    if _pool is None:
        _pool = redis.ConnectionPool.from_url(
            settings.REDIS_URL,
            encoding="utf-8",
            decode_responses=True,
            max_connections=50,
        )
    return _pool


async def get_redis() -> redis.Redis:
    return redis.Redis(connection_pool=get_redis_pool())


class CacheService:
    """Redis cache with JSON serialization."""

    def __init__(self, redis_client: redis.Redis):
        self.r = redis_client
        self.default_ttl = settings.REDIS_CACHE_TTL

    async def get(self, key: str) -> Any | None:
        data = await self.r.get(key)
        if data is None:
            return None
        try:
            return orjson.loads(data)
        except Exception:
            return data

    async def set(self, key: str, value: Any, ttl: int | None = None) -> None:
        serialized = orjson.dumps(value).decode()
        await self.r.set(key, serialized, ex=ttl or self.default_ttl)

    async def delete(self, key: str) -> None:
        await self.r.delete(key)

    async def delete_pattern(self, pattern: str) -> int:
        keys = []
        async for key in self.r.scan_iter(match=pattern, count=100):
            keys.append(key)
        if keys:
            return await self.r.delete(*keys)
        return 0

    async def increment(self, key: str, ttl: int | None = None) -> int:
        val = await self.r.incr(key)
        if val == 1 and ttl:
            await self.r.expire(key, ttl)
        return val

    async def get_or_set(self, key: str, factory, ttl: int | None = None) -> Any:
        cached = await self.get(key)
        if cached is not None:
            return cached
        value = await factory() if callable(factory) else factory
        await self.set(key, value, ttl)
        return value


async def get_cache() -> CacheService:
    r = await get_redis()
    return CacheService(r)
