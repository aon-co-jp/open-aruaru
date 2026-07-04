use anyhow::Result;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor, transport::smtp::authentication::Credentials,
};
use crate::config::SmtpConfig;

pub async fn send_login_email(cfg: &SmtpConfig, to: &str, url: &str) -> Result<()> {
    let body = format!(
        "以下のリンクをクリックしてログインしてください（有効期限: 15分）:\n\n{}\n\n\
         このメールに心当たりがない場合は無視してください。\n\n— aruaru-easyweb",
        url
    );
    let msg = Message::builder()
        .from(cfg.from.parse()?)
        .to(to.parse()?)
        .subject("【aruaru-easyweb】ログインリンク")
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;

    let creds = Credentials::new(cfg.username.clone(), cfg.password.clone());
    AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)?
        .port(cfg.port)
        .credentials(creds)
        .build()
        .send(msg)
        .await?;
    Ok(())
}
