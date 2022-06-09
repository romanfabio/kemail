use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct User {
    #[serde(rename="_id")]
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>
}