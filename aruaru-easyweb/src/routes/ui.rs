use poem::{
    handler, get,
    web::Data,
    EndpointExt, Route, Result,
};
use crate::{db::Pool, settings};

#[handler]
async fn index(Data(pool): Data<&Pool>) -> Result<poem::Response> {
    // セットアップ未完了ならウィザードへ
    if !settings::is_setup_completed(pool).await {
        return Ok(poem::Response::builder()
            .status(poem::http::StatusCode::FOUND)
            .header("Location", "/setup")
            .finish());
    }
    Ok(poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/dashboard.html")))
}

#[handler]
async fn login_page(Data(pool): Data<&Pool>) -> Result<poem::Response> {
    if !settings::is_setup_completed(pool).await {
        return Ok(poem::Response::builder()
            .status(poem::http::StatusCode::FOUND)
            .header("Location", "/setup")
            .finish());
    }
    Ok(poem::Response::builder()
        .status(poem::http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/login.html")))
}

pub fn ui_routes(pool: Pool) -> impl poem::Endpoint {
    Route::new()
        .at("/",      get(index))
        .at("/login", get(login_page))
        .data(pool)
}
