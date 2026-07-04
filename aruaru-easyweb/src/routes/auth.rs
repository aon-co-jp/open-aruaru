use poem::{
    handler, get, post,
    web::{Data, Form, Query},
    EndpointExt, Route, Response, Result,
    http::StatusCode,
    session::Session,
};
use serde::Deserialize;
use crate::{auth, config::AppConfig, db::Pool};

#[derive(Deserialize)]
struct LoginForm { email: String }

#[derive(Deserialize)]
struct VerifyQuery { token: String }

#[handler]
async fn login(
    Data(pool): Data<&Pool>,
    Data(cfg):  Data<&AppConfig>,
    Form(f):    Form<LoginForm>,
) -> Result<Response> {
    auth::send_magic_link(pool, &f.email, &cfg.app_url, cfg.token_expiry_minutes, &cfg.smtp)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Response::builder().status(StatusCode::OK)
        .body("ログインリンクを送信しました。メールをご確認ください。"))
}

#[handler]
async fn verify(
    Data(pool): Data<&Pool>,
    Query(q):   Query<VerifyQuery>,
    session:    &Session,
) -> Result<Response> {
    match auth::verify_token(pool, &q.token).await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?
    {
        Some(uid) => {
            session.set("user_id", uid);
            Ok(Response::builder().status(StatusCode::FOUND).header("Location", "/").finish())
        }
        None => Ok(Response::builder().status(StatusCode::UNAUTHORIZED)
            .body("リンクが無効または期限切れです")),
    }
}

#[handler]
async fn logout(session: &Session) -> Response {
    session.purge();
    Response::builder().status(StatusCode::FOUND).header("Location", "/login").finish()
}

pub fn auth_routes(pool: Pool, cfg: AppConfig) -> impl poem::Endpoint {
    Route::new()
        .at("/login",  post(login))
        .at("/verify", get(verify))
        .at("/logout", post(logout))
        .data(pool)
        .data(cfg)
}
