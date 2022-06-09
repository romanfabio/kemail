use std::collections;

use actix_web::{post, Responder, HttpResponse, web, cookie::Cookie};
use serde::{Serialize, Deserialize};
use crate::model::User;
use mongodb::{Client, Collection, bson::doc};
use jsonwebtoken::{EncodingKey, Header};

#[derive(Debug, Serialize, Deserialize)]
enum Domain {
    #[serde(rename="ke.com")]
    Standard
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterDTO {
    username: String,
    domain: Domain,
    password: String,
    first_name: Option<String>,
    last_name: Option<String>
}

impl From<RegisterDTO> for User {
    fn from(mut r: RegisterDTO) -> Self {
        r.username.push('@');
        r.username.push_str(&serde_json::to_value(&r.domain).unwrap().as_str().unwrap());
        User {
            email: r.username,
            password: r.password,
            first_name: r.first_name,
            last_name: r.last_name
        }
    }
}

#[post("/register")]
async fn register(mongo: web::Data<Client>, user: web::Json<RegisterDTO>) -> impl Responder {
    let mut user : User = user.0.into();

    match bcrypt::hash(&user.password, 10) {
        Ok(hash) => user.password = hash,
        Err(e) => {
            eprintln!("{}",e);
            return HttpResponse::InternalServerError().finish()
        }
    }

    let collection : Collection<User> = mongo.database("storage").collection("users");
    if let Err(e) = collection.insert_one(user, None).await {
        eprintln!("{}",e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[derive(Debug,Serialize,Deserialize)]
struct Claims<'a> {
    email: &'a str
}

#[derive(Debug,Serialize,Deserialize)]
struct LoginDTO {
    email: String,
    password: String
}

#[derive(Serialize,Deserialize)]
struct LoginResponse {
    token: String
}

#[post("/login")]
async fn login(mongo: web::Data<Client>, user: web::Json<LoginDTO>)-> impl Responder {
    let user = user.0;

    let jwt = match authenticate(&mongo, &user.email, &user.password).await {
        Ok(jwt) => jwt,
        Err(x) => match x.t {
            ErrorType::ServerError => return HttpResponse::InternalServerError().json(x),
            ErrorType::Unauthorized => return HttpResponse::Unauthorized().json(x)
        }
    };

    HttpResponse::Ok().json(LoginResponse{token:jwt})
}

#[derive(Serialize)]
enum ErrorType {
    #[serde(rename="UNAUTHORIZED")]
    Unauthorized,
    #[serde(rename="SERVER_ERROR")]
    ServerError
}
#[derive(Serialize)]
struct ErrorResponse {
    #[serde(rename="type")]
    t: ErrorType,
    message: &'static str
}

impl ErrorResponse {
    fn unauthorized() -> ErrorResponse {
        ErrorResponse { t: ErrorType::Unauthorized, message: "invalid username/password" }
    }

    fn server_error() -> ErrorResponse {
        ErrorResponse { t: ErrorType::ServerError, message: "internal server error" }
    }
}

async fn authenticate(mongo: &Client, email: &str, password: &str) -> Result<String, ErrorResponse> {

    let collection : Collection<User> = mongo.database("storage").collection("users");
    let hash = match collection.find_one(doc!{"_id": email}, None).await {
        Ok(Some(u)) => u.password,
        Ok(None) => return Err(ErrorResponse::unauthorized()),
        Err(e) => {
            eprintln!("{}",e);
            return Err(ErrorResponse::server_error())
        }
    };

    match bcrypt::verify(password, &hash) {
        Ok(true) => (),
        Ok(false) => return Err(ErrorResponse::unauthorized()),
        Err(e) => {
            eprintln!("{}",e);
            return Err(ErrorResponse::server_error())
        }
    }

    let claims = Claims{ email: email};
    let jwt = match jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(std::env::var("SECRET").unwrap().as_bytes())) {
        Ok(jwt) => jwt,
        Err(e) => {
            eprintln!("{}",e);
            return Err(ErrorResponse::server_error())
        }
    };

    Ok(jwt)
}