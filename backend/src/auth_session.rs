use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use actix_session::{Session, SessionExt};
use std::future::{ready, Ready};

pub struct AuthSession {
    pub account_id: String,
}

impl FromRequest for AuthSession {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();
        match session.get::<String>("session_id") {
            Ok(Some(account_id)) => ready(Ok(AuthSession { account_id })),
            _ => ready(Err(ErrorUnauthorized("Unauthorized")))
        }
    }
}

pub fn verify_session(session: &Session) -> Result<String, Error> {
    session.get::<String>("session_id")
        .map_err(|_| ErrorUnauthorized("Session error"))?
        .ok_or_else(|| ErrorUnauthorized("Unauthorized"))
}

pub fn invalidate_session(session: &Session) {
    session.remove("session_id");
}