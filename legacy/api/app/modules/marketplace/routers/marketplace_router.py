from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session

router = APIRouter()


@router.get("")
async def list_marketplace(db: AsyncSession = Depends(get_session)):
    return []


@router.get("/{listing_id}")
async def get_listing(listing_id: str, db: AsyncSession = Depends(get_session)):
    return {"message": "Marketplace coming soon", "listing_id": listing_id}
