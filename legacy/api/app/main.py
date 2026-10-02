from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.core.config import get_settings
from app.core.logger import setup_logging, get_logger
from app.core.cache.redis import get_redis

# Import all models BEFORE anything else — fixes SQLAlchemy relationship resolution
import app.core.db.base  # noqa: F401
from app.modules.strategies.models.strategy import Strategy, StrategyVersion, StrategyCopy
from app.modules.backtest.models.backtest_job import BacktestJob, CandleData
from app.modules.tournaments.models.tournament import Tournament, TournamentEntry
from app.modules.marketplace.models.listing import MarketplaceListing, Purchase
from app.modules.billing.models.subscription import Subscription, UsageTracking
from app.modules.users.models.user import User, RefreshToken
from app.modules.rbac.models.role import Role, Permission, UserRole, role_permissions_table

from app.api.middleware.cors import setup_cors
from app.api.middleware.error_handler import setup_error_handlers
from app.api.middleware.request_context import RequestContextMiddleware
from app.api.middleware.rate_limit import RateLimitMiddleware
from app.api.v1.router import api_router

settings = get_settings()
logger = get_logger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    setup_logging()
    logger.info("edgearena_starting", version=settings.APP_VERSION, env=settings.ENVIRONMENT)

    # Verify Redis
    try:
        r = await get_redis()
        await r.ping()
        logger.info("redis_connected")
    except Exception as e:
        logger.warning("redis_unavailable", error=str(e))

    yield

    # Shutdown
    from app.core.db.session import engine
    await engine.dispose()
    logger.info("edgearena_stopped")


app = FastAPI(
    title=settings.APP_NAME,
    version=settings.APP_VERSION,
    docs_url="/docs" if settings.DEBUG else None,
    redoc_url="/redoc" if settings.DEBUG else None,
    openapi_url="/openapi.json" if settings.DEBUG else None,
    lifespan=lifespan,
)

# Middleware (order matters — outermost first)
setup_cors(app)
app.add_middleware(RateLimitMiddleware)
app.add_middleware(RequestContextMiddleware)
setup_error_handlers(app)

# Routers
app.include_router(api_router, prefix=settings.API_V1_PREFIX)


@app.get("/health", tags=["system"])
async def health():
    return {"status": "ok", "service": "edgearena-api", "version": settings.APP_VERSION}


@app.get("/health/ready", tags=["system"])
async def readiness():
    checks = {}
    try:
        r = await get_redis()
        await r.ping()
        checks["redis"] = "ok"
    except Exception:
        checks["redis"] = "unavailable"

    from sqlalchemy import text
    from app.core.db.session import async_session_factory
    try:
        async with async_session_factory() as session:
            await session.execute(text("SELECT 1"))
        checks["database"] = "ok"
    except Exception:
        checks["database"] = "unavailable"

    healthy = all(v == "ok" for v in checks.values())
    return {"status": "ready" if healthy else "degraded", "checks": checks}
