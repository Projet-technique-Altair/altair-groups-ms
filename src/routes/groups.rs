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
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use axum::http::HeaderMap;
use crate::services::extractor::extract_caller;

use crate::{
    error::AppError,
    models::{api::ApiResponse, group::Group, member::GroupMember, assignments::GroupLab},
    state::AppState,
};

use axum::extract::Query;
use serde::Deserialize;


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
        state.groups_service.list_groups_for_user(caller.user_id).await?
    };

    Ok(Json(ApiResponse::success(groups)))
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
    let is_member = state.groups_service.is_member(group_id, caller.user_id).await?;

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
        .update_group(group_id, payload.name, payload.description)
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
    let is_member = state.groups_service.is_member(group_id, caller.user_id).await?;

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
) -> Result<Json<ApiResponse<Vec<Uuid>>>, AppError> {

    let caller = extract_caller(&headers)?;

    let existing_group = state.groups_service.get_group_by_id(group_id).await?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_owner = caller.user_id == existing_group.creator_id;
    let is_member = state.groups_service.is_member(group_id, caller.user_id).await?;

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
    Query(params): Query<AccessLabQuery>,
) -> Result<Json<ApiResponse<bool>>, AppError> {

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
    Query(params): Query<AccessStarpathQuery>,
) -> Result<Json<ApiResponse<bool>>, AppError> {

    let allowed = state
        .groups_service
        .user_has_access_to_starpath(params.user_id, params.starpath_id)
        .await?;

    Ok(Json(ApiResponse::success(allowed)))
}


// ======================================================
// Payloads
// ======================================================

#[derive(Deserialize)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateGroupPayload {
    pub name: String,
    pub description: Option<String>,
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
pub struct AccessLabQuery {
    pub user_id: Uuid,
    pub lab_id: Uuid,
}

#[derive(Deserialize)]
pub struct AccessStarpathQuery {
    pub user_id: Uuid,
    pub starpath_id: Uuid,
}