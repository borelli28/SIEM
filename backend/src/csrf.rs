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

    // Generate the CSRF token and cookie
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
            .path("/")
            .finish();

        // Calculate expiration time and store the token
        let expiration = SystemTime::now() + Duration::from_secs(MINUTES_20.try_into().unwrap());
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(form_id.to_string(), (cookie_value, expiration));

        Ok((csrf_token, cookie))
    }

    // Validate the CSRF cookie and ensure form ID is valid
    pub fn validate_token(&self, cookie: &str, form_id: &str) -> bool {
        let tokens = self.tokens.lock().unwrap();
        if let Some((stored_cookie, expiration)) = tokens.get(form_id) {
            if stored_cookie != cookie || SystemTime::now() > *expiration {
                return false; // Token or session expired
            }
        } else {
            return false; // No matching token found for the form ID
        }

        // If we reach here, it means the cookie is valid and not expired
        true
    }

    pub fn clean_expired_tokens(&self) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, (_, expiration)| expiration > &mut SystemTime::now());
    }
}

// CSRF Validator
pub async fn csrf_validator(req: &HttpRequest, csrf: &CsrfMiddleware) -> Result<(), Error> {
    // Extract CSRF cookie
    let cookie = req.cookie("csrf_token").map(|c| c.value().to_string());
    println!("CSRF Cookie from request: {:?}", cookie);

    // Extract form ID
    let form_id = req.headers().get("X-Form-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            error!("Missing Form ID from headers");
            ErrorForbidden("Missing Form ID")
        })?;

    // Validate cookie
    if cookie.is_none() {
        error!("Missing CSRF cookie");
        return Err(ErrorForbidden("Missing CSRF cookie"));
    }

    let cookie_value = cookie.unwrap();

    // Validate the CSRF cookie with your logic
    if csrf.validate_token(&cookie_value, form_id) {
        return Ok(()); // Validation passed
    }

    error!("CSRF token validation failed for form ID: {}", form_id);
    Err(ErrorForbidden("Invalid CSRF token"))
}