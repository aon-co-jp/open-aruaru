use poem::{handler, get, web::Data, Route, Response, http::StatusCode, Result};
use crate::db::Pool;

#[handler]
async fn index() -> Response {
    Response::builder().status(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/dashboard.html"))
}
#[handler]
async fn terminal_page() -> Response {
    Response::builder().status(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/terminal.html"))
}
#[handler]
async fn filemgr_page() -> Response {
    Response::builder().status(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/filemgr.html"))
}

pub fn ui_routes(pool: Pool) -> Route {
    Route::new()
        .at("/",          get(index))
        .at("/terminal",  get(terminal_page))
        .at("/files",     get(filemgr_page))
}
