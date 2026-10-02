import uuid
from fastapi import APIRouter, Depends, Query
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import get_current_user
from app.modules.users.models.user import User
from app.modules.backtest.schemas.backtest_schema import (
    CreateBacktestRequest,
    BacktestResponse,
    BacktestListItem,
)
from app.modules.backtest.services.backtest_service import BacktestService

router = APIRouter()


@router.post("", response_model=BacktestResponse, status_code=201)
async def create_backtest(
    data: CreateBacktestRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    job = await svc.create(user, data)
    return BacktestResponse.model_validate(job)


@router.get("", response_model=list[BacktestListItem])
async def list_backtests(
    limit: int = Query(50, le=100),
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    return [BacktestListItem.model_validate(j) for j in await svc.list_user(user.id, limit)]


@router.get("/{backtest_id}", response_model=BacktestResponse)
async def get_backtest(
    backtest_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    return BacktestResponse.model_validate(await svc.get(backtest_id, user.id))


@router.post("/{backtest_id}/cancel", response_model=BacktestResponse)
async def cancel_backtest(
    backtest_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    return BacktestResponse.model_validate(await svc.cancel(backtest_id, user.id))


@router.get("/{backtest_id}/trades")
async def get_trades(
    backtest_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    return await svc.get_trades(backtest_id, user.id)


@router.get("/{backtest_id}/equity")
async def get_equity(
    backtest_id: uuid.UUID,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = BacktestService(db)
    return await svc.get_equity(backtest_id, user.id)
