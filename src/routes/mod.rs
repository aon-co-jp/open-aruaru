use poem::Route;

pub mod api;
pub mod pages;
pub mod recovery_api;

pub fn routes() -> Route {
    Route::new()
        .nest("/api", api::api_routes())
        .nest("/api/recovery", recovery_api::recovery_routes())
        .nest("/", pages::page_routes())
}
