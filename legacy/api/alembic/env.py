from logging.config import fileConfig
from sqlalchemy import engine_from_config, pool
from alembic import context
from app.core.config import get_settings
from app.core.db.base import Base

# Import all models to register them with Base.metadata
from app.modules.users.models.user import User, RefreshToken
from app.modules.rbac.models.role import Role, Permission, UserRole, role_permissions_table
from app.modules.strategies.models.strategy import Strategy, StrategyVersion, StrategyCopy
from app.modules.backtest.models.backtest_job import BacktestJob, CandleData
from app.modules.tournaments.models.tournament import Tournament, TournamentEntry
from app.modules.marketplace.models.listing import MarketplaceListing, Purchase
from app.modules.billing.models.subscription import Subscription, UsageTracking

settings = get_settings()
config = context.config
sync_url = settings.DATABASE_URL.replace("+asyncpg", "+psycopg")
config.set_main_option("sqlalchemy.url", sync_url)

if config.config_file_name is not None:
    fileConfig(config.config_file_name)

target_metadata = Base.metadata


def run_migrations_offline():
    url = config.get_main_option("sqlalchemy.url")
    context.configure(url=url, target_metadata=target_metadata, literal_binds=True)
    with context.begin_transaction():
        context.run_migrations()


def run_migrations_online():
    connectable = engine_from_config(
        config.get_section(config.config_ini_section, {}),
        prefix="sqlalchemy.",
        poolclass=pool.NullPool,
    )
    with connectable.connect() as connection:
        context.configure(connection=connection, target_metadata=target_metadata)
        with context.begin_transaction():
            context.run_migrations()


if context.is_offline_mode():
    run_migrations_offline()
else:
    run_migrations_online()
