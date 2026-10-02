from fastapi import APIRouter
from app.modules.auth.routers.auth_router import router as auth_router
from app.modules.strategies.routers.strategy_router import router as strategy_router
from app.modules.backtest.routers.backtest_router import router as backtest_router
from app.modules.tournaments.routers.tournament_router import router as tournament_router
from app.modules.leaderboard.routers.leaderboard_router import router as leaderboard_router
from app.modules.marketplace.routers.marketplace_router import router as marketplace_router
from app.modules.billing.routers.billing_router import router as billing_router
from app.modules.admin.routers.admin_router import router as admin_router

api_router = APIRouter()

api_router.include_router(auth_router, prefix="/auth", tags=["auth"])
api_router.include_router(strategy_router, prefix="/strategies", tags=["strategies"])
api_router.include_router(backtest_router, prefix="/backtests", tags=["backtests"])
api_router.include_router(tournament_router, prefix="/tournaments", tags=["tournaments"])
api_router.include_router(leaderboard_router, prefix="/leaderboard", tags=["leaderboard"])
api_router.include_router(marketplace_router, prefix="/marketplace", tags=["marketplace"])
api_router.include_router(billing_router, prefix="/billing", tags=["billing"])
api_router.include_router(admin_router, prefix="/admin", tags=["admin"])
