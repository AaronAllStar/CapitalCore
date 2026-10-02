from celery import Celery
from app.core.config import get_settings

settings = get_settings()

celery_app = Celery(
    "edgearena",
    broker=settings.CELERY_BROKER_URL,
    backend=settings.CELERY_RESULT_BACKEND,
)

celery_app.conf.update(
    task_serializer="json",
    accept_content=["json"],
    result_serializer="json",
    timezone="UTC",
    enable_utc=True,
    task_track_started=True,
    task_time_limit=settings.BACKTEST_TIMEOUT_SECONDS,
    task_soft_time_limit=settings.BACKTEST_TIMEOUT_SECONDS - 30,
    worker_prefetch_multiplier=1,
    worker_max_tasks_per_child=50,
    task_acks_late=True,
    task_reject_on_worker_lost=True,
    task_default_queue="default",
    task_routes={
        "app.core.queue.tasks.backtest_*": {"queue": "backtest"},
        "app.core.queue.tasks.tournament_*": {"queue": "tournament"},
        "app.core.queue.tasks.ranking_*": {"queue": "ranking"},
        "app.core.queue.tasks.notification_*": {"queue": "notification"},
    },
)

celery_app.autodiscover_tasks(["app.core.queue.tasks"])
