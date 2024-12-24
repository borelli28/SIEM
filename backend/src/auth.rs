use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use futures::future::{err, ok, Ready};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use actix_session::Session;
use uuid::Uuid;

const SESSION_DURATION_MINUTES: u64 = 20;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub account_id: String,
    pub user_agent: String,
    pub expires: SystemTime,
}

// RwLock = Multiple readers & only one writer
pub type SessionStore = Arc<RwLock<HashMap<String, SessionData>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub account_id: String,
}

impl FromRequest for AuthSession {
    type Error = Error;
    type Future = Ready<Result<AuthSession, Error>>;

    fn from_request(req: &HttpRequest) -> Self::Future {
        let session = req.get_session();
        let session_store = req.app_data::<SessionStore>().unwrap().clone();

        match session.get::<String>("session_id") {
            Ok(Some(session_id)) => {
                // Acquire a read lock on the session store
                let store = session_store.read().unwrap();
                if let Some(session_data) = store.get(&session_id) {
                    // Check if the session has not expired
                    if session_data.expires > SystemTime::now() {
                        return ok(AuthSession { account_id: session_data.account_id.clone() });
                    }
                }
            },
            _ => {} // No session_id found or error occurred
        }
        err(ErrorUnauthorized("Unauthorized"))  // No valid session was found
    }
}

fn update_session(
    session_store: &SessionStore,
    session: &Session,
    session_data: &SessionData,
    old_session_id: Option<String>
) -> Result<(), Error> {
    let new_session_id = Uuid::new_v4().to_string();

    let mut store = session_store.write().unwrap();
    if let Some(old_id) = old_session_id {
        store.remove(&old_id);
    }
    store.insert(new_session_id.clone(), session_data.clone());
    session.insert("session_id", new_session_id)?;
    session.renew();

    Ok(())
}

pub fn create_session(req: &HttpRequest, session: &Session, account_id: &str) -> Result<(), Error> {
    let session_store = req.app_data::<SessionStore>().unwrap().clone();
    let user_agent = req.headers().get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    let session_data = SessionData {
        account_id: account_id.to_string(),
        user_agent,
        expires: SystemTime::now() + Duration::from_secs(SESSION_DURATION_MINUTES * 60),
    };

    update_session(&session_store, session, &session_data, None)
}

pub fn verify_session(req: &HttpRequest, session: &Session) -> Result<String, Error> {
    let session_store = req.app_data::<SessionStore>().unwrap().clone();

    if let Ok(Some(session_id)) = session.get::<String>("session_id") {
        let store = session_store.read().unwrap();
        if let Some(session_data) = store.get(&session_id) {
            // Regenerate session ID and update expiration
            if session_data.expires > SystemTime::now() {
                let mut new_session_data = session_data.clone();
                new_session_data.expires = SystemTime::now() + Duration::from_secs(SESSION_DURATION_MINUTES * 60);
                drop(store); // Release the read lock before calling update_session
                update_session(&session_store, session, &new_session_data, Some(session_id))?;
                return Ok(new_session_data.account_id);
            }
        }
    }

    Err(ErrorUnauthorized("Unauthorized"))
}

pub fn invalidate_session(req: &HttpRequest, session: &Session) {
    if let Ok(Some(session_id)) = session.get::<String>("session_id") {
        let session_store = req.app_data::<SessionStore>().unwrap().clone();
        session_store.write().unwrap().remove(&session_id);
    }
    session.purge();
}