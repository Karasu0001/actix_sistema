use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use tera::{Tera, Context};
use sqlx::PgPool;

use crate::services::home_service;
use crate::models::usuario::UserPayload; // 🔥 Importamos el modelo del usuario

// ❌ Controlador para la vista 404 integrada con el Layout Dinámico
pub async fn not_found(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>,
    req: HttpRequest, // 🔥 Agregamos el request para extraer la sesión
) -> HttpResponse {
    
    // 1. Intentamos extraer al usuario (inyectado por tu middleware)
    let user = match req.extensions().get::<UserPayload>() {
        Some(u) => u.clone(),
        None => {
            // Si un usuario NO logueado intenta entrar a una ruta que no existe,
            // lo más seguro es mandarlo al login en lugar de mostrar el layout roto.
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    // 2. Obtenemos el Menú Dinámico real usando el id_perfil del usuario
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil)
        .await
        .unwrap_or_default();
    
    // 3. Renderizamos la vista hija (error/404.html)
    let ctx = Context::new();
    let contenido = tmpl.render("error/404.html", &ctx)
        .unwrap_or_else(|e| format!("Error en vista 404: {}", e));

    // 4. Armamos el Layout Final con los datos dinámicos
    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Página no encontrada");
    layout_ctx.insert("user_nombre", &user.nombre_completo); // 🔥 Dinámico
    layout_ctx.insert("user_iniciales", &user.iniciales);    // 🔥 Dinámico
    layout_ctx.insert("user_email", &user.email);            // 🔥 Dinámico
    layout_ctx.insert("breadcrumbs_placeholder", "Error / 404"); 
    
    // 🔥 Pasamos las variables maestras
    layout_ctx.insert("datos_menu", &datos_menu); 
    layout_ctx.insert("content", &contenido);

    // Renderizamos usando tu función centralizada
    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);
    
    // ⚠️ IMPORTANTE: Devolvemos NotFound (Código 404) pero con nuestra vista bonita
    HttpResponse::NotFound()
        .content_type("text/html; charset=utf-8")
        .body(render)
}