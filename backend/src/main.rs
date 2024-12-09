use actix_web::{web, App, HttpServer, HttpResponse, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        .service(
            web::scope("/backend")
                .route("/", web::get().to(index))
        )
    })
    .bind(("127.0.0.1", 4200)).expect("Failed to bind to address").run().await
}