import uuid
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload
from app.modules.rbac.models.role import (
    Role,
    UserRole,
    Permission,
    RoleEnum,
    PermissionEnum,
    role_permissions_table,
)


class RoleService:
    def __init__(self, db: AsyncSession):
        self.db = db

    async def get_user_roles(self, user_id: uuid.UUID) -> list[Role]:
        result = await self.db.execute(
            select(UserRole)
            .options(selectinload(UserRole.role).selectinload(Role.permissions))
            .where(UserRole.user_id == user_id)
        )
        return [ur.role for ur in result.scalars().all()]

    async def has_role(self, user_id: uuid.UUID, role: RoleEnum) -> bool:
        roles = await self.get_user_roles(user_id)
        return any(r.name == role.value for r in roles)

    async def has_permission(self, user_id: uuid.UUID, permission: PermissionEnum) -> bool:
        roles = await self.get_user_roles(user_id)
        for role in roles:
            if any(p.name == permission.value for p in role.permissions):
                return True
        return False

    async def assign_role(self, user_id: uuid.UUID, role_name: str) -> UserRole:
        result = await self.db.execute(select(Role).where(Role.name == role_name))
        role = result.scalar_one_or_none()
        if not role:
            raise ValueError(f"Role '{role_name}' not found")

        existing = await self.db.execute(
            select(UserRole).where(
                UserRole.user_id == user_id,
                UserRole.role_id == role.id,
            )
        )
        if existing.scalar_one_or_none():
            raise ValueError(f"User already has role '{role_name}'")

        user_role = UserRole(user_id=user_id, role_id=role.id)
        self.db.add(user_role)
        await self.db.flush()
        return user_role

    async def remove_role(self, user_id: uuid.UUID, role_name: str) -> None:
        result = await self.db.execute(
            select(UserRole)
            .join(Role)
            .where(UserRole.user_id == user_id, Role.name == role_name)
        )
        user_role = result.scalar_one_or_none()
        if user_role:
            await self.db.delete(user_role)
            await self.db.flush()

    async def seed_defaults(self) -> None:
        """Create default roles and permissions if they don't exist."""
        # Create permissions
        for perm in PermissionEnum:
            existing = await self.db.execute(
                select(Permission).where(Permission.name == perm.value)
            )
            if not existing.scalar_one_or_none():
                self.db.add(Permission(name=perm.value, description=perm.value.replace(":", " ").title()))
        await self.db.flush()

        # Create roles with permissions
        role_perms = {
            RoleEnum.USER.value: [
                PermissionEnum.STRATEGY_CREATE,
                PermissionEnum.STRATEGY_READ,
                PermissionEnum.STRATEGY_UPDATE,
                PermissionEnum.STRATEGY_DELETE,
                PermissionEnum.BACKTEST_CREATE,
                PermissionEnum.BACKTEST_READ,
                PermissionEnum.BACKTEST_CANCEL,
                PermissionEnum.TOURNAMENT_JOIN,
                PermissionEnum.MARKETPLACE_BUY,
                PermissionEnum.MARKETPLACE_SELL,
            ],
            RoleEnum.MODERATOR.value: [
                PermissionEnum.TOURNAMENT_CREATE,
                PermissionEnum.TOURNAMENT_MANAGE,
                PermissionEnum.ADMIN_STRATEGIES,
                PermissionEnum.ADMIN_TOURNAMENTS,
            ],
            RoleEnum.ADMIN.value: list(PermissionEnum),
        }

        for role_name, perms in role_perms.items():
            result = await self.db.execute(select(Role).where(Role.name == role_name))
            role = result.scalar_one_or_none()
            if not role:
                role = Role(name=role_name, description=f"{role_name.title()} role")
                self.db.add(role)
                await self.db.flush()

            # Assign permissions
            perm_result = await self.db.execute(
                select(Permission).where(Permission.name.in_([p.value for p in perms]))
            )
            role.permissions = list(perm_result.scalars().all())

        await self.db.flush()
