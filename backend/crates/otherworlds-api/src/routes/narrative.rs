//! Routes for the Narrative Orchestration bounded context.

use axum::Router;

use crate::state::AppState;

/// Returns the router for the narrative context.
///
/// Routes will be added as command and query endpoints are implemented.
pub fn router() -> Router<AppState> {
    Router::new()
}
