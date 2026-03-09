use axum::{
    routing::{delete, get},
    Router,
};

use crate::state::AppState;

use crate::routes::{
    groups::{
        add_member, assign_lab, assign_starpath, create_group, delete_group, get_group_by_id,
        list_groups, list_labs, list_members, list_starpaths, remove_member, unassign_lab,
        unassign_starpath, update_group, my_groups,
    },
    health::health,
};

pub mod groups;
pub mod health;

//// AJOUTER UN TRUC SUR LABS POUR L'AJOUTER DANS UN GROUP À LA CRÉATION

pub fn init_routes() -> Router<AppState> {
    Router::new()
        // Health
        .route("/health", get(health))
        // Groups CRUD
        .route("/groups", get(list_groups).post(create_group)) //faire sécu quand on aura mis public/privé
        .route("/mygroups", get(my_groups))
        .route("/groups/:id", get(get_group_by_id).put(update_group).delete(delete_group),
        )
        // Members
        .route("/groups/:id/members", get(list_members).post(add_member)) //uniquement le creator
        .route("/groups/:id/members/:user_id", delete(remove_member))
        // Labs assignments
        .route("/groups/:id/labs", get(list_labs).post(assign_lab))
        .route("/groups/:id/labs/:lab_id", delete(unassign_lab))
        // Starpaths assignments
        .route("/groups/:id/starpaths", get(list_starpaths).post(assign_starpath))
        .route("/groups/:id/starpaths/:starpath_id", delete(unassign_starpath),
        )
}
