/**
 * @file group — core group domain model.
 *
 * @remarks
 * This module defines the primary data structures representing a Group
 * within the Altaïr platform.
 *
 * A group is a central entity that allows creators to:
 *
 *  - Organize learners
 *  - Assign labs and starpaths
 *  - Structure collaborative learning environments
 *
 * The module separates database representation from API exposure:
 *
 *  - `GroupRow`
 *      → პირდაპირ mapping of a database row (used by SQLx)
 *      → Includes all persisted fields as stored in the database
 *
 *  - `Group`
 *      → Public-facing model used in API responses and internal logic
 *      → Mirrors the database structure for simplicity and consistency
 *
 * Conversion strategy:
 *
 *  - `TryFrom<GroupRow> for Group`
 *      → Provides a controlled transformation layer
 *      → Allows validation or transformation logic to be added later
 *      → Returns an `AppError` to integrate with the global error system
 *
 * Field overview:
 *
 *  - `group_id`   → Unique identifier of the group
 *  - `creator_id` → Owner of the group (used for access control)
 *  - `name`       → Display name of the group
 *  - `description`→ Optional description for context
 *  - `created_by` → User who created the group (can differ from owner in future evolutions)
 *  - `created_at` → Creation timestamp (used for sorting and auditing)
 *
 * Design considerations:
 *
 *  - Strong typing with UUIDs ensures safe identification across services
 *  - Explicit timestamps support auditing and chronological ordering
 *  - Separation of concerns keeps persistence and API layers decoupled
 *
 * Used in:
 *
 *  - Group CRUD operations
 *  - Group listing and dashboards
 *  - Access control checks (owner vs member vs admin)
 *
 * @packageDocumentation
 */
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, FromRow)]
pub struct GroupRow {
    pub group_id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub group_id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
}

impl TryFrom<GroupRow> for Group {
    type Error = AppError;

    fn try_from(row: GroupRow) -> Result<Self, Self::Error> {
        Ok(Group {
            group_id: row.group_id,
            creator_id: row.creator_id,
            name: row.name,
            description: row.description,
            status: row.status,
            created_by: row.created_by,
            created_at: row.created_at,
        })
    }
}
