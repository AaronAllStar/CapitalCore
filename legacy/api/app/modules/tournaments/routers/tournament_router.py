import uuid
from fastapi import APIRouter, Depends, Query
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.db.session import get_session
from app.api.deps import get_current_user, require_role
from app.modules.users.models.user import User
from app.modules.rbac.models.role import RoleEnum
from app.modules.tournaments.schemas.tournament_schema import (
    CreateTournamentRequest,
    TournamentResponse,
    JoinTournamentRequest,
    TournamentEntryResponse,
    LeaderboardEntry,
)
from app.modules.tournaments.services.tournament_service import TournamentService

router = APIRouter()


@router.get("", response_model=list[TournamentResponse])
async def list_tournaments(
    status: str | None = Query(None),
    limit: int = Query(50, le=100),
    db: AsyncSession = Depends(get_session),
):
    svc = TournamentService(db)
    tournaments = await svc.list_all(status=status, limit=limit)
    return [TournamentResponse(
        **{k: getattr(t, k) for k in TournamentResponse.model_fields if hasattr(t, k)},
        participant_count=t.participant_count,
    ) for t in tournaments]


@router.post("", response_model=TournamentResponse, status_code=201)
async def create_tournament(
    data: CreateTournamentRequest,
    user: User = Depends(require_role(RoleEnum.MODERATOR)),
    db: AsyncSession = Depends(get_session),
):
    svc = TournamentService(db)
    t = await svc.create(user, data)
    return TournamentResponse(
        **{k: getattr(t, k) for k in TournamentResponse.model_fields if hasattr(t, k)},
        participant_count=t.participant_count,
    )


@router.get("/{tournament_id}", response_model=TournamentResponse)
async def get_tournament(tournament_id: uuid.UUID, db: AsyncSession = Depends(get_session)):
    svc = TournamentService(db)
    t = await svc.get(tournament_id)
    return TournamentResponse(
        **{k: getattr(t, k) for k in TournamentResponse.model_fields if hasattr(t, k)},
        participant_count=t.participant_count,
    )


@router.post("/{tournament_id}/join", response_model=TournamentEntryResponse, status_code=201)
async def join_tournament(
    tournament_id: uuid.UUID,
    data: JoinTournamentRequest,
    user: User = Depends(get_current_user),
    db: AsyncSession = Depends(get_session),
):
    svc = TournamentService(db)
    entry = await svc.join(tournament_id, user, data)
    return TournamentEntryResponse.model_validate(entry)


@router.post("/{tournament_id}/start", response_model=TournamentResponse)
async def start_tournament(
    tournament_id: uuid.UUID,
    user: User = Depends(require_role(RoleEnum.MODERATOR)),
    db: AsyncSession = Depends(get_session),
):
    svc = TournamentService(db)
    t = await svc.start(tournament_id)
    return TournamentResponse(
        **{k: getattr(t, k) for k in TournamentResponse.model_fields if hasattr(t, k)},
        participant_count=t.participant_count,
    )


@router.post("/{tournament_id}/finalize", response_model=TournamentResponse)
async def finalize_tournament(
    tournament_id: uuid.UUID,
    user: User = Depends(require_role(RoleEnum.MODERATOR)),
    db: AsyncSession = Depends(get_session),
):
    svc = TournamentService(db)
    t = await svc.finalize(tournament_id)
    return TournamentResponse(
        **{k: getattr(t, k) for k in TournamentResponse.model_fields if hasattr(t, k)},
        participant_count=t.participant_count,
    )


@router.get("/{tournament_id}/leaderboard", response_model=list[LeaderboardEntry])
async def get_leaderboard(tournament_id: uuid.UUID, db: AsyncSession = Depends(get_session)):
    svc = TournamentService(db)
    return await svc.get_leaderboard(tournament_id)
