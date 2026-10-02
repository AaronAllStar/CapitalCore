"""Task registrations — actual implementations live in worker processes."""
from app.core.queue.celery import celery_app

# Re-export tasks so autodiscovery finds them
# Import task modules to register them with celery
from app.workers.tasks import backtest_tasks  # noqa: F401
from app.workers.tasks import tournament_tasks  # noqa: F401
from app.workers.tasks import ranking_tasks  # noqa: F401
