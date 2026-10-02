from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import get_current_user
from app.modules.users.models.user import User
from app.modules.billing.models.subscription import PLANS

router = APIRouter()


@router.get("/plans")
async def list_plans():
    return PLANS


@router.get("/usage")
async def get_usage(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    return {
        "plan": user.plan,
        "limits": PLANS.get(user.plan, PLANS["free"])["limits"],
        "usage": {
            "strategies": user.total_backtests,
            "backtests_this_month": 0,
            "copies_today": 0,
            "ai_messages_today": 0,
        },
    }
