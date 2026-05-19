use crate::services::extractor::extract_caller;
/**
 * @file groups — HTTP routes for group management.
 *
 * @remarks
 * Defines all REST endpoints related to Groups:
 *
 *  - Group lifecycle (list, create, update, delete)
 *  - Membership management (add/remove/list members)
 *  - Resource assignments (labs & starpaths)
 *  - Access checks (labs & starpaths)
 *
 * Each handler:
 *
 *  - Extracts caller identity via headers (`extract_caller`)
 *  - Applies RBAC rules (admin, owner, member)
 *  - Delegates business logic to `GroupsService`
 *  - Returns standardized responses (`ApiResponse`)
 *
 * Key characteristics:
 *
 *  - Centralized authorization checks per route
 *  - Clear separation: HTTP layer vs business logic
 *  - Consistent error handling via `AppError`
 *  - Supports both user-scoped and admin-level access
 *
 * @packageDocumentation
 */
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        api::ApiResponse,
        assignments::{GroupLab, GroupStarpath},
        group::Group,
        member::GroupMember,
    },
    state::AppState,
};

use serde::{Deserialize, Serialize};

// ======================================================
// GET /groups (public)
// ======================================================
pub async fn list_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<Group>>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    let groups = if is_admin {
        state.groups_service.list_groups().await?
    } else {
        state
            .groups_service
            .list_groups_for_user(caller.user_id)
            .await?
    };

    Ok(Json(ApiResponse::success(groups)))
}

pub async fn list_groups_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AdminGroupsQuery>,
) -> Result<Json<ApiResponse<PaginatedGroups>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    if !is_admin {
        return Err(AppError::Forbidden(
            "Admin role is required to list all groups".into(),
        ));
    }

    let limit = params.limit.unwrap_or(200).clamp(1, 500);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = state
        .groups_service
        .list_groups_admin(params.q, limit, offset)
        .await?;

    Ok(Json(ApiResponse::success(PaginatedGroups {
        items,
        total,
        limit,
        offset,
    })))
}

pub async fn list_admin_user_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Group>>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    if !is_admin {
        return Err(AppError::Forbidden(
            "Admin role is required to inspect user groups".into(),
        ));
    }

    let groups = state.groups_service.list_groups_for_user(user_id).await?;
    Ok(Json(ApiResponse::success(groups)))
}

pub async fn get_admin_group_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::services::groups_service::AdminGroupDetail>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    if !is_admin {
        return Err(AppError::Forbidden(
            "Admin role is required to inspect group detail".into(),
        ));
    }

    let detail = state
        .groups_service
        .get_admin_group_detail(group_id)
        .await?;
    Ok(Json(ApiResponse::success(detail)))
}

pub async fn update_group_status_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<UpdateGroupStatusPayload>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    if !is_admin {
        return Err(AppError::Forbidden(
            "Admin role is required to update group status".into(),
        ));
    }

    let group = state
        .groups_service
        .update_group_status(group_id, payload.status.trim())
        .await?;

    Ok(Json(ApiResponse::success(group)))
}

// ==========================
// GET /mygroups (creator's groups only)
// ==========================
pub async fn my_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<Group>>>, AppError> {
    let caller = extract_caller(&headers)?;

    let groups = state.groups_service.my_groups(caller.user_id).await?;
    Ok(Json(ApiResponse::success(groups)))
}

// ======================================================
// GET /groups/:id (public)
// ======================================================
pub async fn get_group_by_id(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;
    let is_member = state
        .groups_service
        .is_member(group_id, caller.user_id)
        .await?;

    if !(is_admin || is_owner || is_member) {
        return Err(AppError::Forbidden(
            "You are not allowed to see this group's labs.".into(),
        ));
    }

    let group = state.groups_service.get_group_by_id(group_id).await?;
    Ok(Json(ApiResponse::success(group)))
}

// ======================================================
// POST /groups (teacher | admin)
// ======================================================
pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGroupPayload>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let caller = extract_caller(&headers)?;

    let group = state
        .groups_service
        .create_group(
            payload.name,
            payload.description,
            payload.language,
            caller.user_id,
            caller.user_id,
        )
        .await?;

    Ok(Json(ApiResponse::success(group)))
}

// ======================================================
// PUT /groups/:id (owner | admin)
// ======================================================
pub async fn update_group(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<UpdateGroupPayload>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to update this group".into(),
        ));
    }

    let group = state
        .groups_service
        .update_group(
            group_id,
            payload.name,
            payload.description,
            payload.language,
        )
        .await?;

    Ok(Json(ApiResponse::success(group)))
}

// ==========================
// DELETE /groups/:id (owner/admin)
// ==========================
pub async fn delete_group(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to delete this group".into(),
        ));
    }

    state.groups_service.delete_group(group_id).await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET /groups/:id/members
// ======================================================
pub async fn list_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<GroupMember>>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to update this group".into(),
        ));
    }

    let members = state.groups_service.list_members(group_id).await?;
    Ok(Json(ApiResponse::success(members)))
}

// ======================================================
// POST /groups/:id/members
// ======================================================
pub async fn add_member(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<AddMemberPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to update this group".into(),
        ));
    }

    state
        .groups_service
        .add_member(group_id, payload.user_id, "member")
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// DELETE /groups/:id/members/:user_id
// ======================================================
pub async fn remove_member(
    State(state): State<AppState>,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to delete this group".into(),
        ));
    }

    state
        .groups_service
        .remove_member(group_id, user_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET /groups/:id/labs
// ======================================================
pub async fn list_labs(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<GroupLab>>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;
    let is_member = state
        .groups_service
        .is_member(group_id, caller.user_id)
        .await?;

    if !(is_admin || is_owner || is_member) {
        return Err(AppError::Forbidden(
            "You are not allowed to see this group's labs.".into(),
        ));
    }

    let labs = state.groups_service.list_labs(group_id).await?;
    Ok(Json(ApiResponse::success(labs)))
}

// ======================================================
// POST /groups/:id/labs
// ======================================================
pub async fn assign_lab(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<AssignLabPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to add labs to this group".into(),
        ));
    }

    state
        .groups_service
        .assign_lab(group_id, payload.lab_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// DELETE /groups/:id/labs/:lab_id
// ======================================================
pub async fn unassign_lab(
    State(state): State<AppState>,
    Path((group_id, lab_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to delete labs in this group".into(),
        ));
    }

    state.groups_service.unassign_lab(group_id, lab_id).await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET /groups/:id/starpaths
// ======================================================
pub async fn list_starpaths(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<GroupStarpath>>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;
    let is_member = state
        .groups_service
        .is_member(group_id, caller.user_id)
        .await?;

    if !(is_admin || is_owner || is_member) {
        return Err(AppError::Forbidden(
            "You are not allowed to see this group's starpathss.".into(),
        ));
    }

    let starpaths = state.groups_service.list_starpaths(group_id).await?;
    Ok(Json(ApiResponse::success(starpaths)))
}

// ======================================================
// POST /groups/:id/starpaths
// ======================================================
pub async fn assign_starpath(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AssignStarpathPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to add starpaths to this group".into(),
        ));
    }

    state
        .groups_service
        .assign_starpath(group_id, payload.starpath_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// DELETE /groups/:id/starpaths/:starpath_id
// ======================================================
pub async fn unassign_starpath(
    State(state): State<AppState>,
    Path((group_id, starpath_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;

    if !is_admin && !is_owner {
        return Err(AppError::Forbidden(
            "You are not allowed to delete starpaths in this group".into(),
        ));
    }

    state
        .groups_service
        .unassign_starpath(group_id, starpath_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET access labs
// ======================================================
pub async fn check_lab_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AccessLabQuery>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    ensure_internal_access(&state, &headers)?;
    let allowed = state
        .groups_service
        .user_has_access_to_lab(params.user_id, params.lab_id)
        .await?;

    Ok(Json(ApiResponse::success(allowed)))
}

// ======================================================
// GET access starpath
// ======================================================
pub async fn check_starpath_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AccessStarpathQuery>,
) -> Result<Json<ApiResponse<bool>>, AppError> {
    ensure_internal_access(&state, &headers)?;
    let allowed = state
        .groups_service
        .user_has_access_to_starpath(params.user_id, params.starpath_id)
        .await?;

    Ok(Json(ApiResponse::success(allowed)))
}

pub async fn list_internal_user_starpaths(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<InternalUserQuery>,
) -> Result<Json<ApiResponse<Vec<Uuid>>>, AppError> {
    ensure_internal_access(&state, &headers)?;

    let starpath_ids = state
        .groups_service
        .list_user_starpath_ids(params.user_id)
        .await?;

    Ok(Json(ApiResponse::success(starpath_ids)))
}

fn ensure_internal_access(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let provided = headers
        .get("x-altair-internal-token")
        .and_then(|value| value.to_str().ok());
    if provided == Some(state.internal_service_token.as_str()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Internal service token is required".into(),
        ))
    }
}

// ======================================================
// Payloads
// ======================================================

#[derive(Deserialize)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateGroupPayload {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
}

#[derive(Deserialize)]
pub struct AddMemberPayload {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct AssignLabPayload {
    pub lab_id: Uuid,
}

#[derive(Deserialize)]
pub struct AssignStarpathPayload {
    pub starpath_id: Uuid,
}

#[derive(Deserialize)]
pub struct UpdateGroupStatusPayload {
    pub status: String,
}

#[derive(Deserialize)]
pub struct AccessLabQuery {
    pub user_id: Uuid,
    pub lab_id: Uuid,
}

#[derive(Deserialize)]
pub struct AccessStarpathQuery {
    pub user_id: Uuid,
    pub starpath_id: Uuid,
}

#[derive(Deserialize)]
pub struct InternalUserQuery {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct AdminGroupsQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedGroups {
    pub items: Vec<Group>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
