import time
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from app.core.cache.redis import get_redis
from app.core.config import get_settings

settings = get_settings()


class RateLimitMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint) -> Response:
        if not request.url.path.startswith("/api/"):
            return await call_next(request)

        client_ip = request.client.host if request.client else "unknown"
        key = f"ratelimit:ip:{client_ip}:{int(time.time() // 60)}"

        try:
            r = await get_redis()
            current = await r.incr(key)
            if current == 1:
                await r.expire(key, 60)
            if current > settings.RATE_LIMIT_DEFAULT:
                return Response(
                    content='{"error":{"code":"RATE_LIMITED","message":"Too many requests"}}',
                    status_code=429,
                    media_type="application/json",
                    headers={"Retry-After": "60"},
                )
        except Exception:
            pass  # Don't block if Redis is down

        return await call_next(request)
