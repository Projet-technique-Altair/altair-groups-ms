#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupRole {
    Owner,
    Admin,
    Teacher,
    Member,
}

impl GroupRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupRole::Owner => "owner",
            GroupRole::Admin => "admin",
            GroupRole::Teacher => "teacher",
            GroupRole::Member => "member",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupMemberRow {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: Uuid,
    pub role: GroupRole,
    pub joined_at: chrono::NaiveDateTime,
}

impl TryFrom<GroupMemberRow> for GroupMember {
    type Error = AppError;

    fn try_from(row: GroupMemberRow) -> Result<Self, Self::Error> {
        let role = match row.role.as_str() {
            "owner" => GroupRole::Owner,
            "admin" => GroupRole::Admin,
            "teacher" => GroupRole::Teacher,
            "member" => GroupRole::Member,
            other => {
                return Err(AppError::Internal(format!(
                    "Invalid group role in DB: {other}"
                )))
            }
        };

        Ok(Self {
            user_id: row.user_id,
            role,
            joined_at: row.joined_at,
        })
    }
}
