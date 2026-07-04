use poem::session::Session;

pub fn email(session: &Session) -> Option<String> {
    session.get("email")
}

pub fn set_email(session: &Session, email: &str) {
    session.set("email", email);
}
