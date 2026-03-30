use actix_web::{web, HttpResponse, HttpRequest, HttpMessage, cookie::{Cookie, time::Duration}};
use tera::{Context, Tera};
use sqlx::PgPool;
use serde::Deserialize;
use serde_json::json;

use crate::services::{home_service, login_service::LoginService};
use crate::models::usuario::UserPayload;

/// 🏠 INDEX (Dashboard protegido - ahora dinámico)
pub async fn index(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    // 🔥 Obtener usuario desde las extensiones (insertado por el middleware)
    let user = match req.extensions().get::<UserPayload>() {
        Some(u) => u.clone(),
        None => {
            // Fallback por si falla el middleware
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }
    };

    //  Ahora todo es dinámico
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil)
        .await
        .unwrap_or_default();

    let contenido = r#"
        <div class="library-card p-4">
            <h2>Bienvenido 🚀</h2>
            <p>Este es el dashboard inicial en Actix Web, totalmente integrado al Layout dinámico.</p>
            <button class="btn-coffee mt-3">Acción Rápida</button>
        </div>
    "#;

    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Inicio");
    layout_ctx.insert("user_nombre", &user.nombre_completo);
    layout_ctx.insert("user_email", &user.email);
    layout_ctx.insert("user_iniciales", &user.iniciales);
    layout_ctx.insert("breadcrumbs_placeholder", "Inicio");
    layout_ctx.insert("datos_menu", &datos_menu);
    layout_ctx.insert("content", contenido);

    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(render)
}

/// 🔐 LOGIN INDEX (Vista de login - HTML que compartiste)
pub async fn login_index() -> HttpResponse {
    // 🔥 Corregimos la ruta apuntando desde la raíz del proyecto
    match std::fs::read_to_string("src/templates/auth/login.html") {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
        Err(e) => {
            // Imprimimos el error real en consola para depurar más fácil
            println!("[ERROR] No se pudo leer login.html: {}", e); 
            HttpResponse::NotFound()
                .body("<h1>Error: No se encontró el archivo login.html. Revisa la ruta.</h1>")
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub correo: String,
    pub password: String,
    pub captcha: String,
}

/// 🔐 LOGIN API (Autenticación)
pub async fn login_api_dispatcher(
    pool: web::Data<PgPool>,
    json: web::Json<LoginRequest>,
) -> HttpResponse {
    let data = json.into_inner();

    println!("\n[API LOGIN] 📨 Solicitud recibida");

    // 1. Validar credenciales
    if data.correo.is_empty() || data.password.is_empty() {
        return HttpResponse::Ok().json(json!({
            "success": false,
            "msg": "Faltan credenciales"
        }));
    }

    // 2. Validar reCAPTCHA
    // if data.captcha.is_empty() || !LoginService::verify_recaptcha(&data.captcha).await {
    //     return HttpResponse::Ok().json(json!({
    //         "success": false,
    //         "msg": "Verificación de reCAPTCHA fallida. Inténtalo de nuevo."
    //     }));
    // }

    // 3. Autenticar usuario
    match LoginService::authenticate_user(&pool, &data.correo, &data.password).await {
        Ok((success, msg, token_opt, _user_data)) => {
            if success {
                if let Some(token) = token_opt {
                    let cookie = Cookie::build("auth_token", token)
                        .path("/")
                        .max_age(Duration::hours(8))
                        .http_only(true)
                        .same_site(actix_web::cookie::SameSite::Lax)
                        .finish();

                    return HttpResponse::Ok()
                        .cookie(cookie)
                        .json(json!({
                            "success": true,
                            "msg": msg
                        }));
                }
            }

            HttpResponse::Ok().json(json!({
                "success": false,
                "msg": msg
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "msg": e
        })),
    }
}

/// 🚪 LOGOUT
pub async fn logout_action() -> HttpResponse {
    println!("[LOGOUT] 🚪 Cerrando sesión del usuario");

    let cookie = Cookie::build("auth_token", "")
        .path("/")
        .max_age(Duration::seconds(0))
        .finish();

    HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", "/login"))
        .finish()
}