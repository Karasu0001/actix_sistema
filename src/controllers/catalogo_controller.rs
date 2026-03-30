use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use serde_json::json;
use crate::services::catalogo_service;

// 🛠️ Función auxiliar compartida para los errores
fn handle_error(e: sqlx::Error) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "success": false,
        "msg": format!("Error de base de datos: {}", e)
    }))
}

// 📌 Controladores individuales (adiós al dispatcher)
pub async fn get_perfiles(pool: web::Data<PgPool>) -> HttpResponse {
    match catalogo_service::get_perfiles(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => handle_error(e),
    }
}

pub async fn get_sexos(pool: web::Data<PgPool>) -> HttpResponse {
    match catalogo_service::get_sexos(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => handle_error(e),
    }
}

pub async fn get_estados(pool: web::Data<PgPool>) -> HttpResponse {
    match catalogo_service::get_estados(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => handle_error(e),
    }
}

pub async fn get_modulos(pool: web::Data<PgPool>) -> HttpResponse {
    match catalogo_service::get_modulos(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => handle_error(e),
    }
}

pub async fn get_menus(pool: web::Data<PgPool>) -> HttpResponse {
    match catalogo_service::get_menus(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => handle_error(e),
    }
}