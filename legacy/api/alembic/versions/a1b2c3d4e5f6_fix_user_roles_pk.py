"""fix user_roles pk — remove extra id column

Revision ID: a1b2c3d4e5f6
Revises: 5ee41508bb3b
Create Date: 2026-07-20 22:30:00.000000

Contexto:
  La migración inicial creó `user_roles` con PK (user_id, role_id, id),
  pero el modelo UserRole solo necesita (user_id, role_id) como clave compuesta.
  La columna `id` extra y los campos de timestamp de Base causan problemas
  en el mapper de SQLAlchemy al resolver la relación User.roles.

  Esta migración:
    1. Elimina la PK compuesta antigua
    2. Elimina la columna `id`, `created_at`, `updated_at` (heredadas de Base)
    3. Crea la PK correcta (user_id, role_id)
"""
from typing import Sequence, Union
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

revision: str = 'a1b2c3d4e5f6'
down_revision: Union[str, None] = '5ee41508bb3b'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # 1. Drop the old composite PK constraint
    op.drop_constraint('pk_user_roles', 'user_roles', type_='primary')

    # 2. Drop the extra columns added by Base (id, created_at, updated_at)
    op.drop_column('user_roles', 'id')
    op.drop_column('user_roles', 'created_at')
    op.drop_column('user_roles', 'updated_at')

    # 3. Create the correct composite PK
    op.create_primary_key('pk_user_roles', 'user_roles', ['user_id', 'role_id'])

    # 4. Add indexes for fast lookups
    op.create_index('ix_user_roles_user_id', 'user_roles', ['user_id'], unique=False)
    op.create_index('ix_user_roles_role_id', 'user_roles', ['role_id'], unique=False)


def downgrade() -> None:
    # Reverse: restore original (broken) structure
    op.drop_constraint('pk_user_roles', 'user_roles', type_='primary')
    op.drop_index('ix_user_roles_user_id', table_name='user_roles')
    op.drop_index('ix_user_roles_role_id', table_name='user_roles')

    op.add_column('user_roles', sa.Column('id', postgresql.UUID(as_uuid=True), nullable=False, server_default=sa.text('gen_random_uuid()')))
    op.add_column('user_roles', sa.Column('created_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False))
    op.add_column('user_roles', sa.Column('updated_at', sa.DateTime(timezone=True), server_default=sa.text('now()'), nullable=False))

    op.create_primary_key('pk_user_roles', 'user_roles', ['user_id', 'role_id', 'id'])
