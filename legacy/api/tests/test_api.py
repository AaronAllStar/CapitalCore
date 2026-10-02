import pytest
from httpx import AsyncClient, ASGITransport
from app.main import app


@pytest.fixture
async def client():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac


@pytest.mark.asyncio
async def test_health(client: AsyncClient):
    res = await client.get("/health")
    assert res.status_code == 200
    assert res.json()["status"] == "ok"


@pytest.mark.asyncio
async def test_register(client: AsyncClient):
    res = await client.post("/api/v1/auth/register", json={
        "email": "testapi@test.com",
        "username": "testapiuser",
        "password": "testpass123",
    })
    assert res.status_code == 201
    data = res.json()
    assert "access_token" in data["tokens"]
    assert data["user"]["email"] == "testapi@test.com"


@pytest.mark.asyncio
async def test_login_invalid(client: AsyncClient):
    res = await client.post("/api/v1/auth/login", json={
        "email": "nope@test.com",
        "password": "wrong",
    })
    assert res.status_code == 401


@pytest.mark.asyncio
async def test_billing_plans(client: AsyncClient):
    res = await client.get("/api/v1/billing/plans")
    assert res.status_code == 200
    data = res.json()
    assert "free" in data
    assert "basic" in data
    assert "premium" in data


@pytest.mark.asyncio
async def test_leaderboard(client: AsyncClient):
    res = await client.get("/api/v1/leaderboard/global")
    assert res.status_code == 200
