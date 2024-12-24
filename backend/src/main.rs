mod global;
mod database;
mod storage;
mod collector;
mod batch_maker;
mod message_queue;
mod rules;
mod alert;
mod schema;
mod handlers;
mod host;
mod log;
mod account;
mod auth;

use actix_web::{web, App, HttpServer};
use crate::collector::LogCollector;
use crate::handlers::{
    index,
    import_log_handler,
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
    login_account_handler
};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_cors::Cors;
use dotenvy::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let collector = web::Data::new(LogCollector::new());
    dotenv().ok();
    let secret_key = env::var("SESSION_SECRET_KEY").expect("SESSION_SECRET_KEY must be set");
    let cookie_key = Key::from(secret_key.as_bytes());

    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), cookie_key.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .build()
            )
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600)
            )
            .service(
                web::scope("/backend")
                    .route("/", web::get().to(index))
                    .service(
                        web::scope("/log")
                            .route("/import", web::post().to(import_log_handler))
                            .route("/all/{account_id}", web::get().to(get_logs_handler))
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
            )
    })
    .bind(("127.0.0.1", 4200))?
    .run()
    .await
}