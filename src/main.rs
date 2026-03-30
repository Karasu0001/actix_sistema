mod controllers;
mod router;
mod render;
mod services;
mod models;
mod middleware;

use actix_web::{App, HttpServer, web, middleware::Logger};
use actix_files as fs;
use render::init_templates;
use sqlx::postgres::PgPoolOptions;
use middleware::auth_middleware::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let tera = init_templates();

    //DB
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL no definida");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await
        .expect("No se pudo conectar a la DB");

    println!("🚀 Servidor iniciado en http://localhost:8080/");

    HttpServer::new(move || {
        App::new()
            .wrap(AuthMiddleware) 
            .wrap(Logger::default())
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(pool.clone())) // Compartir el pool de DB
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .configure(router::config)
    })
        .bind(("127.0.0.1", 8080))?
        .run().await
}
