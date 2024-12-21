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
    get_logs_handler
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let collector = web::Data::new(LogCollector::new());

    HttpServer::new(move || {
        App::new()
            .app_data(collector.clone())
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
            )
    })
    .bind(("127.0.0.1", 4200))?
    .run()
    .await
}