use actix_web::web;
mod handler;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    use actix_web::{HttpServer, App};
    dotenv::dotenv().ok();

    let client = mongodb::Client::with_uri_str(std::env::var("MONGODB")
        .expect("Missing MONGODB env")).await.unwrap();


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(handler::register)
    }).bind(("0.0.0.0", 6969))?.run().await
}
