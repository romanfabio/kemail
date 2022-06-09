use actix_web::{post, Responder, HttpResponse, web};
use serde::{Serialize, Deserialize};
use crate::model::User;
use mongodb::{Client, Collection};

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