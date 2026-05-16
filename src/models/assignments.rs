/**
 * @file assignments — group resource assignment models (labs & starpaths).
 *
 * @remarks
 * This module defines the data structures used to represent associations between
 * groups and their assigned resources:
 *
 *  - Labs (`GroupLab`)
 *  - Starpaths (`GroupStarpath`)
 *
 * It separates database-level representations (`*Row`) from API-facing models
 * to ensure a clean boundary between persistence and exposure layers.
 *
 * Design details:
 *
 *  - `GroupLabRow` and `GroupStarpathRow`
 * - → Direct mappings of database rows (used by SQLx)
 * - → Include both `group_id` and resource identifiers
 *
 *  - `GroupLab` and `GroupStarpath`
 * - → Simplified structures exposed to the API
 * - → Only include the resource identifier (lab_id / starpath_id)
 * - → Avoid leaking unnecessary internal data (like group_id redundancy)
 *
 *  - `From<Row>` implementations
 * - → Provide automatic conversion from database models to API models
 * - → Keep transformation logic centralized and reusable
 *
 * This design ensures:
 *
 *  - Clear separation between database schema and API contracts
 *  - Minimal payloads returned to clients
 *  - Easier evolution of internal data structures without breaking the API
 *
 * Used in:
 *
 *  - Group assignment endpoints (assign/remove/list labs & starpaths)
 *  - Gateway responses when fetching group content
 *
 * @packageDocumentation
 */
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct GroupLabRow {
    pub lab_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupLab {
    pub lab_id: Uuid,
}

impl From<GroupLabRow> for GroupLab {
    fn from(row: GroupLabRow) -> Self {
        Self { lab_id: row.lab_id }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupStarpathRow {
    pub starpath_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStarpath {
    pub starpath_id: Uuid,
}

impl From<GroupStarpathRow> for GroupStarpath {
    fn from(row: GroupStarpathRow) -> Self {
        Self {
            starpath_id: row.starpath_id,
        }
    }
}
