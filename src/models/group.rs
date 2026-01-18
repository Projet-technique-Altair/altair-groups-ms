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
    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub group_id: Uuid,
    pub creator_id: Uuid,
    pub name: String,
    pub description: Option<String>,
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
            created_by: row.created_by,
            created_at: row.created_at,
        })
    }
}
