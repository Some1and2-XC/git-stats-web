// Mod for handling the auth front-end endpoints
// This includes structs etc

use actix_web::{cookie::time::Date, web::{self, Data}};
use serde::Deserialize;
use sqlx::{query, query_as, types::chrono::NaiveDateTime, FromRow, Sqlite, SqlitePool};

#[derive(Deserialize, Debug, FromRow)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    pub username: String,
    pub credits: Option<i64>,
    pub date_created: Option<NaiveDateTime>,
    pub last_accessed: Option<NaiveDateTime>,
}

impl User {
    fn from_signup_form_data(data: &SignupFormData) -> Self {
        Self {
            id: None,
            email: data.email.clone(),
            username: data.username.clone(),
            credits: None,
            date_created: None,
            last_accessed: None,
        }
    }
}

pub async fn login_handler(db: Data<SqlitePool>, info: web::Form<LoginFormData>) -> String {

    let form_data = info.into_inner();

    let user = query_as::<_, User>("SELECT * FROM Users WHERE email=$1;")
        .bind(&form_data.email)
        .fetch_one(&**db)
        .await.unwrap()
        ;

    // let res = query("SELECT * FROM Users WHERE email=$1;");
    return format!("Form Data: `{:?}`\nUser Data: `{:?}`", form_data, user);
}

#[derive(Deserialize, Debug, FromRow)]
pub struct SignupFormData {
    pub email: String,
    pub username: String,
    pub password: String,
    pub password2: String,
    pub remember: Option<String>,
}

pub async fn signup_handler(db: Data<SqlitePool>, info: web::Form<SignupFormData>) -> String {

    let form_data = info.into_inner();

    if form_data.password != form_data.password2 {
        return "Invalid passwords (your passwords don't match)".to_string();
    }


    let user = User::from_signup_form_data(&form_data);

    let result = query(
        "INSERT INTO Users (email, username)
        VALUES ($1, $2)")
        .bind(user.email)
        .bind(user.username)
        .execute(&**db)
        .await.unwrap()
        ;

    return format!("Got some stuff! `{:?}` with `{}` rows affected!", form_data, result.rows_affected());
}
