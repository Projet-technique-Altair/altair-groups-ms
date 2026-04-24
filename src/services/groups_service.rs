/**
 * @file groups_service — business logic for group management.
 *
 * @remarks
 * This service handles all core operations related to Groups in Altaïr:
 *
 *  - Group lifecycle (create, read, update, delete)
 *  - Membership management (users & roles)
 *  - Resource assignments (labs & starpaths)
 *  - Access checks (membership and resource access)
 *
 * It acts as the bridge between:
 *
 *  - The database (PostgreSQL via SQLx)
 *  - The HTTP layer (handlers)
 *
 * Key characteristics:
 *
 *  - Direct SQL queries (no ORM) for control and performance
 *  - Strong typing with UUIDs
 *  - Idempotent assignments (`ON CONFLICT DO NOTHING`)
 *  - Explicit error handling via `AppError`
 *
 * Access control:
 *
 *  - Helper methods (`is_member`, `user_has_access_*`) are used
 *    by handlers and gateway to enforce permissions.
 *
 * @packageDocumentation
 */
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        assignments::{GroupLab, GroupLabRow, GroupStarpath, GroupStarpathRow},
        group::{Group, GroupRow},
        member::{GroupMember, GroupMemberRow},
    },
};

#[derive(Clone)]
pub struct GroupsService {
    db: PgPool,
}

impl GroupsService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ========= GROUPS ==========

    pub async fn list_groups(&self) -> Result<Vec<Group>, AppError> {
        let rows = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            FROM groups
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(Group::try_from).collect()
    }

    pub async fn list_groups_admin(
        &self,
        query: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Group>, i64), AppError> {
        let query_pattern = query
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value));

        let rows = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            FROM groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR description ILIKE $1)
            ORDER BY created_at DESC
            LIMIT $2
            OFFSET $3
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM groups
            WHERE ($1::TEXT IS NULL OR name ILIKE $1 OR description ILIKE $1)
            "#,
        )
        .bind(query_pattern.as_deref())
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter()
            .map(Group::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(|items| (items, total))
    }

    pub async fn list_groups_for_user(&self, user_id: Uuid) -> Result<Vec<Group>, AppError> {
        let rows = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT DISTINCT
                g.group_id,
                g.creator_id,
                g.name,
                g.description,
                g.created_by,
                g.created_at
            FROM groups g
            LEFT JOIN group_members gm
                ON g.group_id = gm.group_id
            WHERE g.creator_id = $1
            OR gm.user_id = $1
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(Group::try_from).collect()
    }

    // ==========================
    // GET /mygroups (creator's groups only)
    // ==========================
    pub async fn my_groups(&self, creator_id: Uuid) -> Result<Vec<Group>, AppError> {
        let rows = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            FROM groups
            WHERE creator_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(creator_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(Group::try_from).collect()
    }

    // ==========================
    // GET /groups_by_id
    // ==========================
    pub async fn get_group_by_id(&self, group_id: Uuid) -> Result<Group, AppError> {
        let row = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            FROM groups
            WHERE group_id = $1
            "#,
        )
        .bind(group_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("Group not found".into()))?;

        Group::try_from(row)
    }

    // ==========================
    // POST /group
    // ==========================
    pub async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        creator_id: Uuid,
        created_by: Uuid,
    ) -> Result<Group, AppError> {
        let row = sqlx::query_as::<_, GroupRow>(
            r#"
            INSERT INTO groups (name, description, creator_id, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(creator_id)
        .bind(created_by)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Group::try_from(row)
    }

    // ==========================
    // PUT /mygroup_id
    // ==========================
    pub async fn update_group(
        &self,
        group_id: Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<Group, AppError> {
        let row = sqlx::query_as::<_, GroupRow>(
            r#"
            UPDATE groups
            SET name = $1,
                description = $2
            WHERE group_id = $3
            RETURNING
                group_id,
                creator_id,
                name,
                description,
                created_by,
                created_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(group_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("Group not found".into()))?;

        Group::try_from(row)
    }

    // ==========================
    // DELETE /group_id
    // ==========================
    pub async fn delete_group(&self, group_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM groups
            WHERE group_id = $1
            "#,
        )
        .bind(group_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Group not found".into()));
        }

        Ok(())
    }

    // ========= MEMBERS =========

    pub async fn list_members(&self, group_id: Uuid) -> Result<Vec<GroupMember>, AppError> {
        let rows = sqlx::query_as::<_, GroupMemberRow>(
            r#"
            SELECT
                group_id,
                user_id,
                role,
                joined_at
            FROM group_members
            WHERE group_id = $1
            ORDER BY joined_at ASC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        rows.into_iter().map(GroupMember::try_from).collect()
    }

    pub async fn add_member(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO group_members (group_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM group_members
            WHERE group_id = $1
              AND user_id = $2
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Member not found".into()));
        }

        Ok(())
    }

    // ========= LAB ASSIGNMENTS =========

    pub async fn list_labs(&self, group_id: Uuid) -> Result<Vec<GroupLab>, AppError> {
        let rows = sqlx::query_as::<_, GroupLabRow>(
            r#"
            SELECT
                group_id,
                lab_id,
                assigned_at,
                due_date
            FROM group_labs
            WHERE group_id = $1
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| GroupLab { lab_id: r.lab_id })
            .collect())
    }

    pub async fn assign_lab(&self, group_id: Uuid, lab_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO group_labs (group_id, lab_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(group_id)
        .bind(lab_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn unassign_lab(&self, group_id: Uuid, lab_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM group_labs
            WHERE group_id = $1
              AND lab_id = $2
            "#,
        )
        .bind(group_id)
        .bind(lab_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Lab not assigned to group".into()));
        }

        Ok(())
    }

    // ========= STARPATH ASSIGNMENTS =========

    pub async fn list_starpaths(&self, group_id: Uuid) -> Result<Vec<GroupStarpath>, AppError> {
        let rows = sqlx::query_as::<_, GroupStarpathRow>(
            r#"
            SELECT
                group_id,
                starpath_id,
                assigned_at
            FROM group_starpaths
            WHERE group_id = $1
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(GroupStarpath::from).collect())
    }

    pub async fn assign_starpath(&self, group_id: Uuid, starpath_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO group_starpaths (group_id, starpath_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(group_id)
        .bind(starpath_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn unassign_starpath(
        &self,
        group_id: Uuid,
        starpath_id: Uuid,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM group_starpaths
            WHERE group_id = $1
              AND starpath_id = $2
            "#,
        )
        .bind(group_id)
        .bind(starpath_id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Starpath not assigned to group".into()));
        }

        Ok(())
    }

    pub async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM group_members
                WHERE group_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }

    // ==========================
    // GET /user_access_lab
    // ==========================
    pub async fn user_has_access_to_lab(
        &self,
        user_id: Uuid,
        lab_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM group_members gm
                JOIN group_labs gl ON gm.group_id = gl.group_id
                WHERE gm.user_id = $1
                AND gl.lab_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(lab_id)
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }

    // ==========================
    // GET /user_access_starpath
    // ==========================
    pub async fn user_has_access_to_starpath(
        &self,
        user_id: Uuid,
        starpath_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM group_members gm
                JOIN group_starpaths gs ON gm.group_id = gs.group_id
                WHERE gm.user_id = $1
                AND gs.starpath_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(starpath_id)
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }
}
