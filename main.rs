use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, http, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Mutex;

#[derive(Clone, Deserialize, Serialize)]
pub struct LoginInput {
    username: String,
    password: String,
}

pub struct AppState {
    db: PgPool,
    cache: Mutex<Vec<LoginInput>>,
}

pub async fn signup(user: web::Json<LoginInput>, data: web::Data<AppState>) -> impl Responder {
    {
        let mut cache = data.cache.lock().unwrap();
        cache.push(user.clone());
    }

    let result = sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
        .bind(&user.username)
        .bind(&user.password)
        .execute(&data.db)
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Account created successfully"),
        Err(_) => HttpResponse::Conflict().body("Username already exists"),
    }
}

pub async fn login(user: web::Json<LoginInput>, data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query("SELECT password FROM users WHERE username = $1")
        .bind(&user.username)
        .fetch_optional(&data.db)
        .await;

    match result {
        Ok(Some(row)) => {
            let password: String = sqlx::Row::get(&row, "password");
            if password == user.password {
                HttpResponse::Ok().body("Login Successful")
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        _ => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

pub async fn get_cache(data: web::Data<AppState>) -> impl Responder {
    let cache = data.cache.lock().unwrap();
    HttpResponse::Ok().json(cache.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = PgPool::connect("postgres://sauravuchiha@localhost/rust_trade")
        .await
        .expect("Database connection failed");

    let data = web::Data::new(AppState {
        db,
        cache: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE]);

        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .route("/signup", web::post().to(signup))
            .route("/login", web::post().to(login))
            .route("/cache", web::get().to(get_cache))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
