mod global;
mod database;
mod collector;
mod batch_maker;
mod message_queue;
mod rules;
mod alert;
mod host;
mod log;
mod account;
mod auth_session;
mod csrf;
mod schema;
mod eql;
mod agent;
mod handlers;
mod cases;
mod case_comments;
mod log_parser;

use crate::collector::LogCollector;
use crate::handlers::{
    index,
    import_log_handler,
    get_query_logs_handler,
    get_alert_handler,
    get_all_alerts_handler,
    delete_alert_handler,
    acknowledge_alert_handler,
    create_host_handler,
    get_host_handler,
    get_all_hosts_handler,
    edit_host_handler,
    delete_host_handler,
    create_rule_handler,
    get_rule_handler,
    get_all_rules_handler,
    edit_rule_handler,
    delete_rule_handler,
    get_logs_handler,
    create_account_handler,
    get_account_handler,
    edit_account_handler,
    delete_account_handler,
    login_account_handler,
    verify_session_handler,
    logout_handler,
    get_csrf_handler,
    csrf_validator_handler,
    register_agent_handler, 
    agent_upload_handler,
    agent_heartbeat_handler,
    create_case_handler,
    get_case_handler,
    get_cases_by_account_handler,
    update_case_handler,
    delete_case_handler,
    add_observable_handler,
    delete_observable_handler,
    add_comment_handler,
    get_case_comments_handler,
    update_comment_handler,
    delete_comment_handler,
    get_logs_wt_cases_handler
};
use crate::csrf::CsrfMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore, config::PersistentSession};
use actix_web::{web, cookie::time::Duration, cookie::Key, App, HttpServer};
use actix_multipart::form::tempfile::TempFileConfig;
use actix_cors::Cors;
use dotenvy::dotenv;
use env_logger;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let collector = web::Data::new(LogCollector::new());
    dotenv().ok();

    env_logger::init();

    let secret_key = env::var("SESSION_SECRET_KEY").expect("SESSION_SECRET_KEY must be set");
    let cookie_key = Key::from(secret_key.as_bytes());

    // Create CSRF middleware
    let csrf = web::Data::new(CsrfMiddleware::new());

    // Start a background task to clean expired tokens
    let csrf_clone = csrf.clone();
    actix_web::rt::spawn(async move {
        loop {
            actix_web::rt::time::sleep(std::time::Duration::from_secs(3600)).await;
            csrf_clone.clean_expired_tokens();
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
            .app_data(csrf.clone())
            .app_data(TempFileConfig::default().directory("./tmp"))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), cookie_key.clone())
                    .cookie_secure(false) // Set to true in PRODUCTION
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Lax)
                    .cookie_name("auth_session".to_string())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::minutes(20))
                    )
                    .build()
            )
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization", "X-Form-ID"])
                    .supports_credentials()
                    .max_age(3600)
            )
            .service(
                web::scope("/backend")
                    .route("/", web::get().to(index))
                    .route("/check-auth", web::get().to(verify_session_handler))
                    .route("/logout", web::post().to(logout_handler))
                    .service(
                        web::scope("/csrf")
                            .route("/", web::get().to(get_csrf_handler))
                            .route("/validate", web::post().to(csrf_validator_handler))
                    )
                    .service(
                        web::scope("/log")
                            .route("/import", web::post().to(import_log_handler))
                            .route("/all/{account_id}", web::get().to(get_logs_handler))
                            .route("/filter", web::get().to(get_query_logs_handler))
                    )
                    .service(
                        web::scope("/alert")
                            .route("/{alert_id}", web::get().to(get_alert_handler))
                            .route("/all/{account_id}", web::get().to(get_all_alerts_handler))
                            .route("/{alert_id}", web::delete().to(delete_alert_handler))
                            .route("/acknowledge/{alert_id}", web::put().to(acknowledge_alert_handler))
                    )
                    .service(
                        web::scope("/host")
                            .route("/{account_id}", web::post().to(create_host_handler))
                            .route("/{host_id}", web::get().to(get_host_handler))
                            .route("/all/{account_id}", web::get().to(get_all_hosts_handler))
                            .route("/{host_id}", web::put().to(edit_host_handler))
                            .route("/{host_id}", web::delete().to(delete_host_handler))
                    )
                    .service(
                        web::scope("/rule")
                            .route("/{account_id}", web::post().to(create_rule_handler))
                            .route("/{rule_id}", web::get().to(get_rule_handler))
                            .route("/all/{account_id}", web::get().to(get_all_rules_handler))
                            .route("/{rule_id}", web::put().to(edit_rule_handler))
                            .route("/{rule_id}", web::delete().to(delete_rule_handler))
                    )
                    .service(
                        web::scope("/account")
                            .route("/", web::post().to(create_account_handler))
                            .route("/login", web::post().to(login_account_handler))
                            .route("/{account_id}", web::get().to(get_account_handler))
                            .route("/{account_id}", web::put().to(edit_account_handler))
                            .route("/{account_id}", web::delete().to(delete_account_handler))
                    )
                    .service(
                        web::scope("/agent")
                            .route("/register", web::post().to(register_agent_handler))
                            .route("/upload", web::post().to(agent_upload_handler))
                            .route("/heartbeat", web::post().to(agent_heartbeat_handler))
                    )
                    .service(
                        web::scope("/case")
                            .route("/{account_id}", web::post().to(create_case_handler))
                            .route("/{case_id}", web::get().to(get_case_handler))
                            .route("/all/{account_id}", web::get().to(get_cases_by_account_handler))
                            .route("/{case_id}", web::put().to(update_case_handler))
                            .route("/{case_id}", web::delete().to(delete_case_handler))
                            .route("/{case_id}/observable", web::post().to(add_observable_handler))
                            .route("/{case_id}/observable", web::delete().to(delete_observable_handler))
                            .route("/{case_id}/comment", web::post().to(add_comment_handler))
                            .route("/{case_id}/comments", web::get().to(get_case_comments_handler))
                            .route("/comment/{comment_id}", web::put().to(update_comment_handler))
                            .route("/comment/{comment_id}", web::delete().to(delete_comment_handler))
                            .route("/logs/{account_id}", web::get().to(get_logs_wt_cases_handler))
                    )
            )
    })
    .bind(("127.0.0.1", 4200))?
    .run()
    .await
}