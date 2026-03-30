use actix_web::{web, HttpResponse, HttpRequest, HttpMessage}; // 🔥 Importamos HttpRequest y HttpMessage
use tera::{Tera, Context};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::services::perfil_service;
use crate::services::{permisos_service, home_service};
use crate::models::permiso::PermisosPerfil;
use crate::models::usuario::UserPayload; // 🔥 Importamos el modelo de usuario

// 📄 Vista Renderizada (Perfiles)
pub async fn index(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>, 
    req: HttpRequest, // 🔥 Agregamos el request para leer el token
) -> HttpResponse {
    
    // 🔥 1. Extraemos el usuario desde el middleware
    let user = match req.extensions().get::<UserPayload>() {
        Some(u) => u.clone(),
        None => {
            // Si la sesión expiró o no existe, al login
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };
    
    // 2. Validamos Permisos usando el ID de perfil REAL del usuario
    let todos_los_permisos = permisos_service::get_permisos_by_perfil(&pool, user.id_perfil)
        .await
        .unwrap_or_default();

    // ⚠️ ATENCIÓN: El ID de módulo 1 asume que es "Perfiles". 
    // Si tu módulo de perfiles tiene otro ID en la base de datos, ajústalo aquí.
    let permisos_usuario = todos_los_permisos.into_iter()
        .find(|p| p.idmodulo == Some(1)) 
        .unwrap_or_else(|| {
            PermisosPerfil {
                idmodulo: Some(1), id: None, idperfil: Some(user.id_perfil), strnombremodulo: None,
                bitagregar: false, biteditar: false, biteliminar: false, bitconsulta: false, bitdetalle: false
            }
        });

    // Validamos que tenga permiso de ver (consulta)
    if !permisos_usuario.bitconsulta {
        return HttpResponse::NotFound().body("No tienes permisos para consultar este módulo");
    }

    // 3. Obtenemos el Menú Dinámico real
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil).await.unwrap_or_default();
    
    // 4. Renderizamos la vista hija (perfil.html)
    let mut ctx = Context::new();
    ctx.insert("permisos", &permisos_usuario);
    ctx.insert("permisos_json", &serde_json::to_string(&permisos_usuario).unwrap());

    let contenido = tmpl.render("seguridad/perfil.html", &ctx).unwrap_or_else(|e| format!("Error en vista: {}", e));

    // 5. Armamos el Layout Final (Totalmente Dinámico)
    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Gestión de Perfiles");
    layout_ctx.insert("user_nombre", &user.nombre_completo); // 👈 Dinámico
    layout_ctx.insert("user_iniciales", &user.iniciales);    // 👈 Dinámico
    layout_ctx.insert("user_email", &user.email);            // 👈 Dinámico
    layout_ctx.insert("breadcrumbs_placeholder", "Seguridad / Perfiles"); 
    
    // 🔥 Pasamos las variables maestras
    layout_ctx.insert("datos_menu", &datos_menu); 
    layout_ctx.insert("content", &contenido);

    // Renderizamos usando tu función centralizada
    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render)
}

// 📦 DTO para recibir datos
#[derive(Deserialize, Serialize)] // Asegúrate de tener Serialize también
#[serde(rename_all = "camelCase")]
pub struct PerfilDTO {
    pub id: Option<i32>,
    pub strNombrePerfil: String,
    pub bitAdministrador: Option<bool>,
}

// 🔍 GET ALL
pub async fn get_all(pool: web::Data<PgPool>) -> HttpResponse {
    match perfil_service::get_all(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

// 🔍 GET BY ID
pub async fn get_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> HttpResponse {
    match perfil_service::get_by_id(&pool, path.into_inner()).await {
        Ok(Some(data)) => HttpResponse::Ok().json(data),
        Ok(None) => HttpResponse::NotFound().body("No encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

// 💾 SAVE (INSERT / UPDATE)
pub async fn save(
    pool: web::Data<PgPool>,
    json: web::Json<PerfilDTO>,
) -> HttpResponse {
    let data = json.into_inner();

    let result = perfil_service::save(
        &pool,
        data.id,
        data.strNombrePerfil,
        data.bitAdministrador.unwrap_or(false),
    )
    .await;

    match result {
        Ok(msg) => HttpResponse::Ok().json(json!({
            "success": true,
            "msg": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "msg": e.to_string()
        })),
    }
} // ✅ ← ESTA LLAVE FALTABA


// ❌ DELETE
pub async fn delete(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> HttpResponse {

    let result = perfil_service::delete(&pool, path.into_inner()).await;

    match result {
        Ok(msg) => HttpResponse::Ok().json(json!({
            "success": true,
            "msg": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "msg": e.to_string()
        })),
    }
}