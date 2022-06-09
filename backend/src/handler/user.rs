use actix_web::{post, Responder, HttpResponse, web};
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)]
struct UserRegisterDTO {
    username: String,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>
}

#[derive(Serialize,Deserialize)]
struct User {
    #[serde(rename="_id")]
    email: String,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>
}

#[post("/register")]
async fn register(db: web::Data<mongodb::Client>, user: web::Json<UserRegisterDTO>) -> impl Responder {
    let mut user = User{email: user.username, password: user.password, first_name: user.first_name, last_name: user.last_name};
    user.email.push_str("@ke.com");

    HttpResponse::Ok()
}