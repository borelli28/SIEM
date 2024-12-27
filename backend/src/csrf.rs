use actix_web::{cookie::Cookie, error::ErrorForbidden, Error, HttpRequest};
use csrf::{ChaCha20Poly1305CsrfProtection, CsrfProtection};
use base64::{engine::general_purpose, Engine};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::RngCore;
use log::error;


const MINUTES_20: i64 = 20 * 60;

pub struct CsrfMiddleware {
    csrf_protection: Arc<ChaCha20Poly1305CsrfProtection>,
    tokens: Mutex<HashMap<String, (String, SystemTime)>>,
}

#[derive(Serialize, Deserialize)]
pub struct CsrfToken {
    token: String,
    form_id: String,
}

impl CsrfMiddleware {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        let csrf_protection = Arc::new(ChaCha20Poly1305CsrfProtection::from_key(key));
        CsrfMiddleware { 
            csrf_protection,
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn generate_token_pair(&self, form_id: &str) -> Result<(CsrfToken, Cookie), csrf::CsrfError> {
        // Generate the token pair
        let (token, cookie) = self.csrf_protection.generate_token_pair(None, MINUTES_20)
            .map_err(|e| {
                error!("Failed to generate CSRF token pair: {:?}", e);
                csrf::CsrfError::InternalError
            })?;

        // Base64 encode the cookie value
        let cookie_value = general_purpose::STANDARD.encode(cookie.value());

        let csrf_token = CsrfToken {
            token: token.b64_string(),
            form_id: form_id.to_string(),
        };

        // Build the CSRF cookie with error handling
        let cookie = Cookie::build("csrf_token", cookie_value.clone())
            .http_only(true)
            .secure(false) // Set to true in PRODUCTION
            .finish();

        // Calculate expiration time and store the token
        let expiration = SystemTime::now() + Duration::from_secs(MINUTES_20.try_into().unwrap());
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(form_id.to_string(), (cookie_value, expiration));

        Ok((csrf_token, cookie))
    }

    pub fn validate_token(&self, token: &str, cookie: &str, form_id: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        if let Some((stored_cookie, expiration)) = tokens.get(form_id) {
            if stored_cookie != cookie || SystemTime::now() > *expiration {
                return false;
            }
        } else {
            return false;
        }

        if let (Ok(parsed_token), Ok(parsed_cookie)) = (
            self.csrf_protection.parse_token(token.as_bytes()),
            self.csrf_protection.parse_cookie(cookie.as_bytes()),
        ) {
            self.csrf_protection.verify_token_pair(&parsed_token, &parsed_cookie)
        } else {
            false
        }
    }

    pub fn clean_expired_tokens(&self) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, (_, expiration)| expiration > &mut SystemTime::now());
    }
}

pub async fn csrf_validator(req: &HttpRequest, csrf: &CsrfMiddleware) -> Result<(), Error> {
    let token = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok());

    let cookie = req.cookie("csrf_token").map(|c| c.value().to_string());

    let form_id = req
        .headers()
        .get("X-Form-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ErrorForbidden("Missing Form ID"))?;

    if let (Some(token), Some(cookie)) = (token, cookie) {
        if csrf.validate_token(token, &cookie, form_id) {
            Ok(())
        } else {
            Err(ErrorForbidden("Invalid CSRF token"))
        }
    } else {
        Err(ErrorForbidden("Missing CSRF token or cookie"))
    }
}