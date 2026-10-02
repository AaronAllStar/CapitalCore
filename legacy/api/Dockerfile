FROM python:3.12-slim AS base

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential libpq-dev curl \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

FROM base AS api
EXPOSE 8000
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000", "--workers", "4", "--loop", "uvloop", "--http", "httptools"]

FROM base AS worker
CMD ["celery", "-A", "app.core.queue.celery", "worker", "--loglevel=info", "--concurrency=4", "-Q", "backtest,tournament,ranking,default"]

FROM base AS beat
CMD ["celery", "-A", "app.core.queue.celery", "beat", "--loglevel=info", "--schedule", "/tmp/celerybeat-schedule"]
