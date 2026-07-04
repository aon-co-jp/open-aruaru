use poem::{handler, post, web::Json, Route, Result, http::StatusCode};
use crate::webserver::recommender::{recommend, Framework};

#[handler]
async fn get_recommendation(Json(req): Json<RecommendReq>) -> Result<Json<serde_json::Value>> {
    let rec = recommend(&req.framework);
    Ok(Json(serde_json::json!({
        "webserver":  rec.webserver,
        "reason":     rec.reason,
        "confidence": rec.confidence,
    })))
}

#[derive(serde::Deserialize)]
struct RecommendReq { framework: Framework }

pub fn webserver_routes(_pool: crate::db::Pool) -> poem::Route {
    Route::new().at("/recommend", post(get_recommendation))
}
