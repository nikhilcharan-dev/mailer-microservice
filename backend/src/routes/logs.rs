use axum::{
    extract::{Extension, Query, State},
    response::Json,
    routing::get,
    Router,
};
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::AppState;
use crate::auth::middleware::AuthIdentity;
use crate::db::logs::{self, LogFilter};
use crate::error::ApiResult;

pub fn router() -> Router<AppState> {
    Router::new().route("/logs", get(list))
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub transport: Option<String>,
    pub template: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<u64>,
}

async fn list(
    State(state): State<AppState>,
    Extension(identity): Extension<AuthIdentity>,
    Query(q): Query<LogQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let page = q.page.unwrap_or(1).max(1);

    let filter = LogFilter {
        user_id: &identity.user_id,
        transport: q.transport.as_deref(),
        template: q.template.as_deref(),
        status: q.status.as_deref(),
    };

    let total = logs::count(&state.db.db, &filter).await?;
    let items = logs::list(&state.db.db, &filter, page, limit).await?;

    let logs_json: Vec<_> = items
        .iter()
        .map(|l| {
            let sent_at = Utc
                .timestamp_millis_opt(l.sent_at.timestamp_millis())
                .single()
                .unwrap_or_else(Utc::now);
            json!({
                "id":             l.id.map(|o| o.to_hex()).unwrap_or_default(),
                "transport_name": l.transport_name,
                "template_name":  l.template_name,
                "to":             l.to,
                "status":         l.status,
                "error":          l.error,
                "duration_ms":    l.duration_ms,
                "sent_at":        sent_at,
            })
        })
        .collect();

    let pages = if limit > 0 {
        (total as i64 + limit - 1) / limit
    } else {
        1
    };

    Ok(Json(json!({
        "logs":  logs_json,
        "total": total,
        "page":  page,
        "pages": pages,
    })))
}
