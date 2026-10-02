from fastapi import APIRouter, Depends, Request
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import get_current_user
from app.modules.users.models.user import User
from app.modules.auth.schemas.auth_schema import (
    RegisterRequest,
    LoginRequest,
    RefreshRequest,
    AuthResponse,
    TokenResponse,
    UserProfile,
    UpdateProfileRequest,
    ChangePasswordRequest,
)
from app.modules.auth.services.auth_service import AuthService

router = APIRouter()


@router.post("/register", response_model=AuthResponse, status_code=201)
async def register(
    data: RegisterRequest,
    request: Request,
    db: AsyncSession = Depends(get_session),
):
    user_agent = request.headers.get("User-Agent")
    ip = request.client.host if request.client else None
    svc = AuthService(db)
    return await svc.register(data, user_agent=user_agent, ip=ip)


@router.post("/login", response_model=AuthResponse)
async def login(
    data: LoginRequest,
    request: Request,
    db: AsyncSession = Depends(get_session),
):
    user_agent = request.headers.get("User-Agent")
    ip = request.client.host if request.client else None
    svc = AuthService(db)
    return await svc.login(data, user_agent=user_agent, ip=ip)


@router.post("/refresh", response_model=TokenResponse)
async def refresh(data: RefreshRequest, db: AsyncSession = Depends(get_session)):
    svc = AuthService(db)
    return await svc.refresh(data.refresh_token)


@router.post("/logout", status_code=204)
async def logout(data: RefreshRequest, db: AsyncSession = Depends(get_session)):
    svc = AuthService(db)
    await svc.logout(data.refresh_token)


@router.post("/logout-all", status_code=200)
async def logout_all(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = AuthService(db)
    count = await svc.logout_all(user.id)
    return {"revoked": count}


@router.get("/me", response_model=UserProfile)
async def get_me(
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = AuthService(db)
    return await svc.get_profile(user.id)


@router.patch("/me", response_model=UserProfile)
async def update_me(
    data: UpdateProfileRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = AuthService(db)
    return await svc.update_profile(user, data)


@router.post("/change-password", status_code=200)
async def change_password(
    data: ChangePasswordRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = AuthService(db)
    await svc.change_password(user, data)
    return {"message": "Password changed successfully"}
