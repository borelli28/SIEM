use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use actix_session::{Session, SessionExt};
use std::time::{Duration, SystemTime};
use std::future::{ready, Ready};

pub struct AuthSession {
    pub account_id: String,
}

impl FromRequest for AuthSession {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();
        match verify_session(&session, &req) {
            Ok(account_id) => ready(Ok(AuthSession { account_id })),
            Err(e) => ready(Err(e)),
        }
    }
}

pub fn verify_session(session: &Session, req: &HttpRequest) -> Result<String, Error> {
    match session.get::<String>("account_id") {
        Ok(Some(account_id)) => {
            // Check session user agent
            let stored_user_agent: Option<String> = session.get("user_agent")
                .map_err(|_| ErrorUnauthorized("Session error"))?;
            let current_user_agent = req.headers().get(actix_web::http::header::USER_AGENT)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            if stored_user_agent.as_deref() != Some(&current_user_agent) {
                return Err(ErrorUnauthorized("Session user agent mismatch"));
            }

            // Check if the last_activity is set
            match session.get::<SystemTime>("last_activity") {
                Ok(Some(last_activity)) => {
                    let session_timeout = Duration::from_secs(20 * 60);
                    // Check if the last activity was more than 20 minutes ago
                    if SystemTime::now().duration_since(last_activity).unwrap_or(Duration::from_secs(0)) > session_timeout {
                        session.purge();
                        return Err(ErrorUnauthorized("Session timeout"));
                    }
                },
                Ok(None) => {
                    // If last_activity is None
                    return Err(ErrorUnauthorized("Unauthorized")); 
                },
                Err(_) => {
                    // Error in retrieving last_activity
                    return Err(ErrorUnauthorized("Session error"));
                }
            }

            session.renew();
            session.insert("last_activity", SystemTime::now())
                .map_err(|_| ErrorUnauthorized("Failed to update session"))?;

            Ok(account_id)
        },
        _ => Err(ErrorUnauthorized("Unauthorized")),
    }
}

pub fn invalidate_session(session: &Session) {
    session.purge();
}