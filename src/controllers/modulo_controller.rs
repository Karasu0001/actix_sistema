use actix_web::{web, HttpResponse, HttpRequest, HttpMessage}; // 🔥 Agregamos HttpRequest y HttpMessage
use tera::{Tera, Context};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::services::modulo_service;
use crate::services::{permisos_service, home_service}; // 🔥 Importamos servicios extra
use crate::models::permiso::PermisosPerfil; // 🔥 Importamos el modelo de permisos
use crate::models::usuario::UserPayload; // 🔥 Importamos la sesión

// 📦 DTO para recibir datos de Módulo del JS (Se queda igual)
#[derive(Deserialize, Serialize)]
pub struct ModuloDTO {
    pub id: Option<i32>,
    #[serde(rename = "strNombreModulo")]
    pub str_nombre_modulo: String,
    #[serde(rename = "nombreMenu")]
    pub nombre_menu: String,
    #[serde(rename = "strRuta")]
    pub str_ruta: Option<String>,
}

// 📦 DTO para recibir datos del Menú (PUT / DELETE) (Se queda igual)
#[derive(Deserialize, Serialize)]
pub struct MenuDTO {
    pub id: i32,
    #[serde(rename = "strNombreMenu")]
    pub str_nombre_menu: Option<String>,
}

// ==========================================
// 📄 VISTA PRINCIPAL (🔥 Ahora 100% Dinámica)
// ==========================================
pub async fn index(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>, // 🔥 Necesitamos el pool
    req: HttpRequest,        // 🔥 Necesitamos el request para la sesión
) -> HttpResponse {
    
    // 1. Extraemos el usuario desde el middleware
    let user = match req.extensions().get::<UserPayload>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Found().append_header(("Location", "/login")).finish();
        }
    };

    // 2. Validamos Permisos (⚠️ OJO: Asumimos que el ID del módulo es 2. Ajusta si es diferente)
    let todos_los_permisos = permisos_service::get_permisos_by_perfil(&pool, user.id_perfil).await.unwrap_or_default();
    let permisos_usuario = todos_los_permisos.into_iter()
        .find(|p| p.idmodulo == Some(2)) 
        .unwrap_or_else(|| {
            PermisosPerfil {
                idmodulo: Some(2), id: None, idperfil: Some(user.id_perfil), strnombremodulo: None,
                bitagregar: false, biteditar: false, biteliminar: false, bitconsulta: false, bitdetalle: false
            }
        });

    if !permisos_usuario.bitconsulta {
        return HttpResponse::NotFound().body("No tienes permisos para consultar este módulo");
    }

    // 3. Obtenemos menús laterales y los menús disponibles para el Dropdown
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil).await.unwrap_or_default();
    let menus_disponibles = modulo_service::get_menus(&pool).await.unwrap_or_default();

    // 4. Renderizamos la vista hija
    let mut ctx = Context::new();
    ctx.insert("permisos", &permisos_usuario);
    ctx.insert("permisos_json", &serde_json::to_string(&permisos_usuario).unwrap());
    ctx.insert("menus_disponibles", &menus_disponibles); // 🔥 Mandamos los menús al HTML

    let contenido = tmpl.render("seguridad/modulo.html", &ctx)
        .unwrap_or_else(|e| format!("Error vista: {}", e));

    // 5. Armamos el Layout Final con datos reales
    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Gestión de Módulos");
    layout_ctx.insert("user_nombre", &user.nombre_completo); // 🔥 Dinámico
    layout_ctx.insert("user_email", &user.email);            // 🔥 Dinámico
    layout_ctx.insert("user_iniciales", &user.iniciales);    // 🔥 Dinámico
    layout_ctx.insert("breadcrumbs_placeholder", "Seguridad / Módulos");
    
    // Pasamos el menú dinámico y el contenido
    layout_ctx.insert("datos_menu", &datos_menu);
    layout_ctx.insert("content", &contenido);

    // Usamos la función render unificada
    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(render)
}

// ==========================================
// 📡 API: MÓDULOS
// ==========================================
pub async fn get_all(pool: web::Data<PgPool>) -> HttpResponse {
    match modulo_service::get_all(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn get_by_id(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    match modulo_service::get_by_id(&pool, path.into_inner()).await {
        Ok(Some(data)) => HttpResponse::Ok().json(data),
        Ok(None) => HttpResponse::NotFound().body("No encontrado"),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn save(pool: web::Data<PgPool>, json: web::Json<ModuloDTO>) -> HttpResponse {
    let data = json.into_inner();

    if data.nombre_menu.trim().is_empty() {
        return HttpResponse::BadRequest().json(json!({ "success": false, "msg": "El Menú Padre es obligatorio." }));
    }

    match modulo_service::save(&pool, data.id, data.str_nombre_modulo, data.nombre_menu, data.str_ruta).await {
        Ok(msg) => HttpResponse::Ok().json(json!({ "success": true, "msg": msg })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "success": false, "msg": e.to_string() })),
    }
}

pub async fn delete(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    match modulo_service::delete(&pool, path.into_inner()).await {
        Ok(msg) => HttpResponse::Ok().json(json!({ "success": true, "msg": msg })),
        Err(_) => HttpResponse::InternalServerError().json(json!({ "success": false, "msg": "No se puede eliminar: el módulo está vinculado a permisos existentes." })),
    }
}

// ==========================================
// 📡 API: MENÚS (Extra)
// ==========================================
pub async fn get_menus(pool: web::Data<PgPool>) -> HttpResponse {
    match modulo_service::get_menus(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().body(format!("{}", e)),
    }
}

pub async fn update_menu(pool: web::Data<PgPool>, json: web::Json<MenuDTO>) -> HttpResponse {
    let data = json.into_inner();
    if let Some(nombre) = data.str_nombre_menu {
        match modulo_service::update_menu(&pool, data.id, nombre).await {
            Ok(msg) => HttpResponse::Ok().json(json!({ "success": true, "msg": msg })),
            Err(e) => HttpResponse::InternalServerError().json(json!({ "success": false, "msg": e.to_string() })),
        }
    } else {
        HttpResponse::BadRequest().json(json!({ "success": false, "msg": "Nombre requerido" }))
    }
}

pub async fn delete_menu(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    match modulo_service::delete_menu(&pool, path.into_inner()).await {
        Ok(msg) => HttpResponse::Ok().json(json!({ "success": true, "msg": msg })),
        Err(_) => HttpResponse::InternalServerError().json(json!({ "success": false, "msg": "No se puede eliminar: el menú está siendo utilizado." })),
    }
}