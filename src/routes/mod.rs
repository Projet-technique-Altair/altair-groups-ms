use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::state::AppState;

use crate::routes::{
    health::health,
    groups::{
        list_groups,
        get_group_by_id,
        create_group,
        update_group,
        delete_group,

        list_members,
        add_member,
        remove_member,

        list_labs,
        assign_lab,
        unassign_lab,

        list_starpaths,
        assign_starpath,
        unassign_starpath,
    },
};

pub mod health;
pub mod groups;

pub fn init_routes() -> Router<AppState> {
    Router::new()
        // Health
        .route("/health", get(health))

        // Groups CRUD
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/:id", get(get_group_by_id).put(update_group).delete(delete_group),
        )

        // Members
        .route("/groups/:id/members", get(list_members).post(add_member))
        .route("/groups/:id/members/:user_id", delete(remove_member),
        )

        // Labs assignments
        .route("/groups/:id/labs", get(list_labs).post(assign_lab))
        .route("/groups/:id/labs/:lab_id", delete(unassign_lab),
        )

        // Starpaths assignments
        .route("/groups/:id/starpaths", get(list_starpaths).post(assign_starpath))
        .route("/groups/:id/starpaths/:starpath_id", delete(unassign_starpath),
        )
}
