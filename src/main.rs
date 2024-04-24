use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use rust_simple_crud::configuration::get_configuration;
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to get configration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres.");

    let db_pool = web::Data::new(connection_pool);

    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(greet))
            .route("/health-check", web::get().to(health_check))
            .route("/subscription", web::post().to(subscribe))
            .route("/get-subscriptions", web::get().to(get_subscriptions))
            .route(
                "/delete-subscriptions/{email}",
                web::delete().to(delete_subscriptions),
            )
            .app_data(db_pool.clone())
    })
    .bind(address)?
    .run()
    .await
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

#[derive(serde::Serialize)]
struct Subscription {
    id: Uuid,
    email: String,
    name: String,
    subscribed_at: DateTime<Utc>,
}

async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_subscriptions(pool: web::Data<PgPool>) -> HttpResponse {
    let value = sqlx::query_as!(Subscription, r"SELECT * FROM subscriptions")
        .fetch_all(pool.as_ref())
        .await
        .expect("Failed to fetch all");

    let body = serde_json::to_string(&value).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
}

async fn delete_subscriptions(req: HttpRequest, pool: web::Data<PgPool>) -> HttpResponse {
    let email = req.match_info().get("email").unwrap();

    match sqlx::query!(
        r#"
   DELETE FROM subscriptions WHERE email = $1
    "#,
        email
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
