use crate::services::groups_service::GroupsService;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub groups_service: GroupsService,
}

impl AppState {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let db = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        Self {
            groups_service: GroupsService::new(db),
        }
    }
}
