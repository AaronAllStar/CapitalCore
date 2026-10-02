import uuid
import enum
from datetime import datetime
from sqlalchemy import String, Integer, Boolean, Date, ForeignKey, Enum as SAEnum, Index
from sqlalchemy import Numeric
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship
from app.core.db.base import Base


class PlanEnum(str, enum.Enum):
    FREE = "free"
    BASIC = "basic"
    PREMIUM = "premium"


class SubscriptionStatus(str, enum.Enum):
    ACTIVE = "active"
    CANCELLED = "cancelled"
    PAST_DUE = "past_due"
    TRIALING = "trialing"


PLANS = {
    "free": {
        "name": "Free", "price": 0,
        "limits": {"strategies": 5, "backtests_per_month": 20, "copies_per_day": 5, "ai_messages_per_day": 10},
    },
    "basic": {
        "name": "Basic", "price": 9,
        "limits": {"strategies": 25, "backtests_per_month": 200, "copies_per_day": 50, "ai_messages_per_day": 100},
    },
    "premium": {
        "name": "Premium", "price": 29,
        "limits": {"strategies": "unlimited", "backtests_per_month": "unlimited", "copies_per_day": "unlimited", "ai_messages_per_day": "unlimited"},
    },
}


class Subscription(Base):
    __tablename__ = "subscriptions"

    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), unique=True, nullable=False
    )
    plan: Mapped[str] = mapped_column(String(20), nullable=False, default="free")
    status: Mapped[str] = mapped_column(
        SAEnum(SubscriptionStatus, native_enum=False), default=SubscriptionStatus.ACTIVE
    )
    stripe_customer_id: Mapped[str | None] = mapped_column(String(255))
    stripe_subscription_id: Mapped[str | None] = mapped_column(String(255))
    current_period_start: Mapped[datetime | None] = mapped_column(nullable=True)
    current_period_end: Mapped[datetime | None] = mapped_column(nullable=True)
    cancel_at_period_end: Mapped[bool] = mapped_column(Boolean, default=False)

    user = relationship("User", back_populates="subscription", lazy="joined")

    @property
    def is_active(self) -> bool:
        return self.status in (SubscriptionStatus.ACTIVE.value, SubscriptionStatus.TRIALING.value)


class UsageTracking(Base):
    __tablename__ = "usage_tracking"

    user_id: Mapped[uuid.UUID] = mapped_column(
        UUID(as_uuid=True), ForeignKey("users.id", ondelete="CASCADE"), nullable=False
    )
    resource: Mapped[str] = mapped_column(String(50), nullable=False)
    count: Mapped[int] = mapped_column(Integer, default=0)
    period: Mapped[datetime] = mapped_column(Date, nullable=False)

    __table_args__ = (
        Index("ix_usage_user_resource_period", "user_id", "resource", "period", unique=True),
    )
