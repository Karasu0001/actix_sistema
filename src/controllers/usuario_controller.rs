use actix_web::{web, HttpResponse, Responder};
use crate::models::usuario::Usuario;
use crate::services::usuario_service::UsuarioService;
use sqlx::PgPool;
use serde_json::json;
use std::collections::HashMap;

pub struct UsuarioController;

impl UsuarioController {
    // --- VISTA HTML ---
    pub async fn index() -> impl Responder {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(include_str!("../templates/usuarios.html"))
    }

    // --- API ENDPOINTS ---

    // GET /api/users (Listar todos o uno por ID)
    pub async fn dispatcher_get(
        pool: web::Data<PgPool>, 
        query: web::Query<HashMap<String, String>>
    ) -> impl Responder {
        if let Some(id_str) = query.get("id") {
            if let Ok(id) = id_str.parse::<i32>() {
                let data = UsuarioService::get_user_by_id(pool.get_ref(), id).await;
                return HttpResponse::Ok().json(data);
            }
        }
        
        let data = UsuarioService::get_all_users(pool.get_ref()).await;
        HttpResponse::Ok().json(data)
    }

    // POST /api/users (Crear)
    pub async fn dispatcher_post(
        pool: web::Data<PgPool>, 
        usuario: web::Json<Usuario>
    ) -> impl Responder {
        // Validación básica manual (similar a validate_form)
        if usuario.usuario.trim().is_empty() || usuario.email.trim().is_empty() {
            return HttpResponse::Ok().json(json!({
                "success": false,
                "msg": "El nombre de usuario y el email son obligatorios",
                "errors": {"usuario": "Campo requerido", "email": "Campo requerido"}
            }));
        }

        let (success, message) = UsuarioService::register_user(pool.get_ref(), usuario.into_inner()).await;

        if !success {
            let mut errors = json!({});
            // Si el error es de duplicado (identificado en el service)
            if message.to_lowercase().contains("registrado") || message.contains("email") {
                errors = json!({"email": message});
            }
            return HttpResponse::Ok().json(json!({
                "success": false,
                "errors": errors,
                "msg": message
            }));
        }

        HttpResponse::Ok().json(json!({"success": true, "msg": message}))
    }

    // PUT /api/users (Actualizar)
    pub async fn dispatcher_put(
        pool: web::Data<PgPool>, 
        usuario: web::Json<Usuario>
    ) -> impl Responder {
        // En el PUT, el ID es obligatorio
        if usuario.id.is_none() {
            return HttpResponse::BadRequest().json(json!({"success": false, "msg": "ID no proporcionado"}));
        }

        let (success, message) = UsuarioService::update_existing_user(pool.get_ref(), usuario.into_inner()).await;
        
        HttpResponse::Ok().json(json!({
            "success": success,
            "msg": message
        }))
    }

    // DELETE /api/users
    pub async fn dispatcher_delete(
        pool: web::Data<PgPool>, 
        body: web::Json<serde_json::Value>
    ) -> impl Responder {
        let user_id = body.get("id").and_then(|v| v.as_i64()).map(|v| v as i32);

        match user_id {
            Some(id) => {
                let (success, message) = UsuarioService::delete_user(pool.get_ref(), id).await;
                HttpResponse::Ok().json(json!({
                    "success": success,
                    "msg": message
                }))
            },
            None => HttpResponse::BadRequest().json(json!({
                "success": false,
                "msg": "ID de usuario no válido"
            }))
        }
    }
}