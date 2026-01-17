use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    state::AppState,
    error::AppError,
    models::{
        api::ApiResponse,
        group::Group,
        member::GroupMember,
    },
};

// ======================================================
// GET /groups (public)
// ======================================================
pub async fn list_groups(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Group>>>, AppError> {
    let groups = state.groups_service.list_groups().await?;
    Ok(Json(ApiResponse::success(groups)))
}

// ======================================================
// GET /groups/:id (public)
// ======================================================
pub async fn get_group_by_id(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = state.groups_service.get_group_by_id(group_id).await?;
    Ok(Json(ApiResponse::success(group)))
}

// ======================================================
// POST /groups (teacher | admin)
// ======================================================
// ======================================================
// POST /groups (teacher | admin)
// ======================================================
pub async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupPayload>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = state
        .groups_service
        .create_group(
            payload.name,
            payload.description,
            payload.creator_id,
            payload.created_by,
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
    Json(payload): Json<UpdateGroupPayload>,
) -> Result<Json<ApiResponse<Group>>, AppError> {
    let group = state
        .groups_service
        .update_group(group_id, payload.name, payload.description)
        .await?;

    Ok(Json(ApiResponse::success(group)))
}

// ======================================================
// DELETE /groups/:id (owner | admin)
// ======================================================
pub async fn delete_group(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    state.groups_service.delete_group(group_id).await?;
    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET /groups/:id/members
// ======================================================
pub async fn list_members(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<GroupMember>>>, AppError> {
    let members = state.groups_service.list_members(group_id).await?;
    Ok(Json(ApiResponse::success(members)))
}

// ======================================================
// POST /groups/:id/members
// ======================================================
pub async fn add_member(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AddMemberPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
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
) -> Result<Json<ApiResponse<()>>, AppError> {
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
) -> Result<Json<ApiResponse<Vec<Uuid>>>, AppError> {
    let labs = state.groups_service.list_labs(group_id).await?;
    Ok(Json(ApiResponse::success(labs)))
}

// ======================================================
// POST /groups/:id/labs
// ======================================================
pub async fn assign_lab(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AssignLabPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
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
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .groups_service
        .unassign_lab(group_id, lab_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// GET /groups/:id/starpaths
// ======================================================
pub async fn list_starpaths(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Uuid>>>, AppError> {
    let starpaths = state.groups_service.list_starpaths(group_id).await?;
    Ok(Json(ApiResponse::success(starpaths)))
}

// ======================================================
// POST /groups/:id/starpaths
// ======================================================
pub async fn assign_starpath(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AssignStarpathPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
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
) -> Result<Json<ApiResponse<()>>, AppError> {
    state
        .groups_service
        .unassign_starpath(group_id, starpath_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// ======================================================
// Payloads
// ======================================================

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateGroupPayload {
    pub name: String,
    pub description: Option<String>,
    pub creator_id: Uuid,
    pub created_by: Uuid,
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
