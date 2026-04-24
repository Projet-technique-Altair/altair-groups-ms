/**
 * @file member — group membership and role management models.
 *
 * @remarks
 * This module defines how users are associated with groups, including their
 * roles and permissions within those groups.
 *
 * It is a core component of the Groups service access control system.
 *
 * Key concepts:
 *
 *  - A user can belong to a group with a specific `GroupRole`
 *  - Roles determine permissions (management, teaching, participation, etc.)
 *  - Membership data is stored in the database and exposed through a clean API model
 *
 * Role system:
 *
 *  - `Owner`
 *      → Full control over the group (creation, deletion, role management)
 *
 *  - `Admin`
 *      → Elevated permissions (manage members, assign resources)
 *
 *  - `Teacher`
 *      → Can manage learning content (labs/starpaths) within the group
 *
 *  - `Member`
 *      → Default role with basic access to assigned content
 *
 * Serialization:
 *
 *  - Roles are serialized using `SCREAMING_SNAKE_CASE` for API consistency
 *  - Internal database values use lowercase strings (`owner`, `admin`, etc.)
 *
 * Data model separation:
 *
 *  - `GroupMemberRow`
 *      → Direct mapping of database rows (SQLx)
 *      → Stores role as a raw `String`
 *
 *  - `GroupMember`
 *      → Strongly-typed API model
 *      → Uses `GroupRole` enum for type safety
 *
 * Conversion strategy:
 *
 *  - `TryFrom<GroupMemberRow> for GroupMember`
 *      → Validates and converts raw role strings into `GroupRole`
 *      → Ensures invalid roles cannot propagate to the application layer
 *      → Returns `AppError` if an unknown role is encountered
 *
 * Utility:
 *
 *  - `GroupRole::as_str()`
 *      → Provides a normalized string representation for persistence or comparisons
 *
 * Design considerations:
 *
 *  - Strong typing prevents invalid role usage across the codebase
 *  - Explicit error handling guards against database inconsistencies
 *  - Clear role hierarchy enables future RBAC extensions
 *
 * Used in:
 *
 *  - Membership management endpoints (add/remove/update members)
 *  - Authorization checks in services and gateway
 *  - Group dashboards and user role displays
 *
 * @packageDocumentation
 */

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

