//! Routes for the Content Authoring bounded context.

use axum::Router;

use crate::state::AppState;

/// Returns the router for the content context.
///
/// Routes will be added as command and query endpoints are implemented.
pub fn router() -> Router<AppState> {
    Router::new()
}
