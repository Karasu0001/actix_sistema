use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use tera::{Tera, Context};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::json;

// 🔥 Añadimos home_service para traer el menú
use crate::services::{permisos_service, home_service}; 
use crate::models::permiso::PermisosPerfil;
use crate::models::usuario::UserPayload;

#[derive(Deserialize)]
pub struct BulkPermisosDTO {
    pub id_perfil: i32,
    pub permisos: Vec<PermisosPerfil>,
}

// 📄 VISTA PRINCIPAL
pub async fn permisos_manager_index(
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

    // 1. Obtenemos todos los permisos
    let todos_los_permisos: Vec<PermisosPerfil> = match permisos_service::get_permisos_by_perfil(&pool, user.id_perfil).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error al cargar: {}", e)),
    };

    // 2. Validamos el permiso de ESTE módulo
    // ⚠️ ATENCIÓN: Puse Some(3) asumiendo que el 3 es "Permisos-Perfil". 
    // Ajusta el número según el ID real en tu tabla de módulos.
    let permisos_modulo = todos_los_permisos.into_iter()
        .find(|p| p.idmodulo == Some(3)) 
        .unwrap_or_else(|| {
            PermisosPerfil {
                idmodulo: Some(3), id: None, idperfil: Some(user.id_perfil), strnombremodulo: None,
                bitagregar: false, biteditar: false, biteliminar: false, bitconsulta: false, bitdetalle: false
            }
        });

    if !permisos_modulo.bitconsulta {
        return HttpResponse::NotFound().body("No tienes permisos para ver este módulo");
    }

    // 3. 🔥 OBTENEMOS EL MENÚ DINÁMICO
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil).await.unwrap_or_default();

    // 4. Renderizamos vista interna
    let mut ctx = Context::new();
    ctx.insert("permisos", &permisos_modulo);
    ctx.insert("permisos_json", &serde_json::to_string(&permisos_modulo).unwrap());

    let contenido = tmpl.render("seguridad/permisos_perfil.html", &ctx)
        .unwrap_or_else(|e| format!("Error vista: {}", e));

    // 5. Armamos el Layout Final
    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Gestión de Permisos");
  layout_ctx.insert("user_nombre", &user.nombre_completo); // 👈 Dinámico
    layout_ctx.insert("user_iniciales", &user.iniciales);    // 👈 Dinámico
    layout_ctx.insert("user_email", &user.email);        // 👈 Dinámico
    layout_ctx.insert("breadcrumbs_placeholder", "Seguridad / Permisos");
    
    // 🔥 PASAMOS LA VARIABLE DEL MENÚ
    layout_ctx.insert("datos_menu", &datos_menu);
    layout_ctx.insert("content", &contenido);

    // Renderizamos usando el render general
    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);

    HttpResponse::Ok().content_type("text/html").body(render)
}

// 🔍 API GET
pub async fn get_permisos_by_perfil(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> HttpResponse {
    let id_perfil = path.into_inner();
    
    // 👇 ¡Aquí está el problema potencial!
    match permisos_service::get_permisos_by_view_perfil(&pool, id_perfil).await { 
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "msg": e.to_string()
        })),
    }
}

// 💾 API POST
pub async fn bulk_update_permisos(
    pool: web::Data<PgPool>,
    payload: web::Json<BulkPermisosDTO>,
) -> HttpResponse {
    let data = payload.into_inner();
    let mut success_count = 0;

    for mut permiso in data.permisos {
        permiso.idperfil = Some(data.id_perfil);
        
        match permisos_service::update_permiso(&pool, permiso).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "msg": format!("Error al actualizar: {}", e)
                }));
            }
        }
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "msg": format!("Se actualizaron {} módulos con éxito", success_count)
    }))
}