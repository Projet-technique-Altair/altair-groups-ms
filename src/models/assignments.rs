use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;

/// ===== LAB ASSIGNMENTS =====

#[derive(Debug, Clone, FromRow)]
pub struct GroupLabRow {
    pub group_id: Uuid,
    pub lab_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupLab {
    pub lab_id: Uuid,
}

impl From<GroupLabRow> for GroupLab {
    fn from(row: GroupLabRow) -> Self {
        GroupLab {
            lab_id: row.lab_id,
        }
    }
}

/// ===== STARPATH ASSIGNMENTS =====

#[derive(Debug, Clone, FromRow)]
pub struct GroupStarpathRow {
    pub group_id: Uuid,
    pub starpath_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupStarpath {
    pub starpath_id: Uuid,
}

impl From<GroupStarpathRow> for GroupStarpath {
    fn from(row: GroupStarpathRow) -> Self {
        GroupStarpath {
            starpath_id: row.starpath_id,
        }
    }
}
