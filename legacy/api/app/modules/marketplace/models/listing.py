import uuid
import enum
from datetime import datetime
from sqlalchemy import String, Integer, Boolean, Text, ForeignKey, Enum as SAEnum
from sqlalchemy import Numeric
from sqlalchemy.dialects.postgresql import UUID, JSONB
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class ListingStatus(str, enum.Enum):
    ACTIVE = "active"
    SOLD = "sold"
    SUSPENDED = "suspended"
    CANCELLED = "cancelled"


class MarketplaceListing(Base):
    __tablename__ = "marketplace_listings"

    strategy_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("strategies.id", ondelete="CASCADE"), nullable=False
    )
    seller_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    price: Mapped[float] = mapped_column(Numeric(10, 2), nullable=False)
    pricing_type: Mapped[str] = mapped_column(String(20), default="one_time")
    description: Mapped[str | None] = mapped_column(Text)
    preview_metrics: Mapped[dict | None] = mapped_column(JSONB)
    status: Mapped[str] = mapped_column(
        SAEnum(ListingStatus, native_enum=False), default=ListingStatus.ACTIVE
    )
    sales_count: Mapped[int] = mapped_column(Integer, default=0)
    avg_rating: Mapped[float | None] = mapped_column(Numeric(3, 2))


class Purchase(Base):
    __tablename__ = "purchases"

    buyer_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id"), nullable=False
    )
    listing_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("marketplace_listings.id"), nullable=False
    )
    amount: Mapped[float] = mapped_column(Numeric(10, 2), nullable=False)
    platform_fee: Mapped[float] = mapped_column(Numeric(10, 2), nullable=False)
    stripe_payment_id: Mapped[str | None] = mapped_column(String(255))
