pub mod projects;
pub mod environments;
pub mod flags;
pub mod flag_environments;
pub mod groups;
pub mod evaluation;
pub mod audit;
pub mod users;
pub mod api_keys;
pub mod webhooks;

use axum::Router;
use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(evaluation::routes::router(state.clone()))
        .merge(projects::routes::router())
        .merge(environments::routes::router())
        .merge(flags::routes::router())
        .merge(flag_environments::routes::router())
        .merge(groups::routes::router())
        .merge(audit::routes::router())
        .merge(users::routes::router(state.clone()))
        .merge(api_keys::routes::router())
        .merge(webhooks::routes::router())
}
