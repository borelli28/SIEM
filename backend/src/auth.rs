use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpMessage};
use actix_web::error::ErrorUnauthorized;
use futures::future::{err, ok, Ready};
use std::time::{Duration, SystemTime};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use serde::{Deserialize, Serialize};
use aes_gcm::aead::{Aead, NewAead};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use actix_session::Session;
use dotenv::dotenv;
use uuid::Uuid;
use rand::Rng;
use std::env;

const SESSION_DURATION_MINUTES: u64 = 20;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub account_id: String,
    pub user_agent: String,
    pub expires: SystemTime,
}

// RwLock = Multiple readers & only one writer
pub type SessionStore = Arc<RwLock<HashMap<String, (Vec<u8>, [u8; 12])>>>;

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
                if let Some((encrypted_data, nonce)) = store.get(&session_id) {
                    // Attempt to decrypt the session data
                    match decrypt_session_data(encrypted_data, nonce) {
                        Ok(session_data) => {
                            // Check if the session has not expired
                            if session_data.expires > SystemTime::now() {
                                return ok(AuthSession { account_id: session_data.account_id });
                            }
                            // If expired, fall through to return Unauthorized
                        },
                        Err(_) => return err(ErrorUnauthorized("Invalid session")),
                    }
                }
            },
            _ => {} // No session_id found or error occurred
        }
        err(ErrorUnauthorized("Unauthorized"))  // No valid session was found
    }
}

pub fn create_session(req: &HttpRequest, session: &Session, account_id: &str) -> Result<(), Error> {
    let session_store = req.app_data::<SessionStore>().unwrap().clone();
    let session_id = Uuid::new_v4().to_string();
    let user_agent = req.headers().get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    let session_data = SessionData {
        account_id: account_id.to_string(),
        user_agent,
        expires: SystemTime::now() + Duration::from_minutes(SESSION_DURATION_MINUTES),
    };

    let (encrypted_data, nonce) = encrypt_session_data(&session_data)?;
    session_store.write().unwrap().insert(session_id.clone(), (encrypted_data, nonce));
    session.insert("session_id", session_id)?;
    session.renew();

    Ok(())
}

pub fn verify_session(req: &HttpRequest, session: &Session) -> Result<String, Error> {
    let session_store = req.app_data::<SessionStore>().unwrap().clone();

    if let Ok(Some(session_id)) = session.get::<String>("session_id") {
        let mut store = session_store.write().unwrap();
        if let Some((encrypted_data, nonce)) = store.get(&session_id) {
            match decrypt_session_data(encrypted_data, nonce) {
                Ok(mut session_data) => {
                    if session_data.expires > SystemTime::now() {
                        // Regenerate session ID and update expiration
                        let new_session_id = Uuid::new_v4().to_string();
                        session_data.expires = SystemTime::now() + Duration::from_minutes(SESSION_DURATION_MINUTES);
                        let (new_encrypted_data, new_nonce) = encrypt_session_data(&session_data)?;
                        store.remove(&session_id);
                        store.insert(new_session_id.clone(), (new_encrypted_data, new_nonce));
                        session.insert("session_id", new_session_id)?;
                        session.renew();

                        return Ok(session_data.account_id);
                    }
                },
                Err(_) => return Err(ErrorUnauthorized("Invalid session")),
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

fn encrypt_session_data(session_data: &SessionData) -> Result<(Vec<u8>, [u8; 12]), Error> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    let nonce_ref = Nonce::from_slice(&nonce);

    let serialized = serde_json::to_vec(session_data)?;
    let encrypted = cipher.encrypt(nonce_ref, serialized.as_ref())
        .map_err(|_| ErrorUnauthorized("Encryption failed"))?;

    Ok((encrypted, nonce))
}

fn decrypt_session_data(encrypted_data: &[u8], nonce: &[u8; 12]) -> Result<SessionData, Error> {
    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce_ref = Nonce::from_slice(nonce);

    let decrypted = cipher.decrypt(nonce_ref, encrypted_data)
        .map_err(|_| ErrorUnauthorized("Decryption failed"))?;

    serde_json::from_slice(&decrypted)
        .map_err(|_| ErrorUnauthorized("Invalid session data"))
}

fn get_encryption_key() -> Result<Key<Aes256Gcm>, Error> {
    dotenv().ok();
    let key_str = env::var("SESSION_SECRET_KEY")
        .map_err(|_| ErrorUnauthorized("Missing SESSION_SECRET_KEY"))?;

    Key::<Aes256Gcm>::from_slice(key_str.as_bytes())
        .map_err(|_| ErrorUnauthorized("Invalid key length"))
}