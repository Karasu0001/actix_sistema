use actix_web::{ web, HttpResponse, HttpRequest, HttpMessage };
use tera::{ Tera, Context };
use sqlx::PgPool;
use serde_json::json;

use crate::services::{ permisos_service, home_service };
use crate::models::permiso::PermisosPerfil;
use crate::models::usuario::UserPayload;

// ==========================================
// 📄 PRINCIPAL 1
// ==========================================

pub async fn p1_1_index(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> HttpResponse {
    modulo_generico_action(
        tmpl,
        pool,
        req,
        "Principal 1 / Submódulo 1.1",
        "principal1",
        "Principal 1.1"
    ).await
}

pub async fn p1_2_index(tmpl: web::Data<Tera>, pool: web::Data<PgPool>, req: HttpRequest,) -> HttpResponse {
    modulo_generico_action(
        tmpl,
        pool,
        req,
        "Principal 1 / Submódulo 1.2",
        "principal1",
        "Principal 1.2"
    ).await
}

// ==========================================
// 📄 PRINCIPAL 2
// ==========================================

pub async fn p2_1_index(tmpl: web::Data<Tera>, pool: web::Data<PgPool>, req: HttpRequest,) -> HttpResponse {
    modulo_generico_action(
        tmpl,
        pool,
        req,
        "Principal 2 / Submódulo 2.1",
        "principal2",
        "Principal 2.1"
    ).await
}

pub async fn p2_2_index(tmpl: web::Data<Tera>, pool: web::Data<PgPool>, req: HttpRequest,) -> HttpResponse {
    modulo_generico_action(
        tmpl,
        pool,
        req,
        "Principal 2 / Submódulo 2.2",
        "principal2",
        "Principal 2.2"
    ).await
}

/// 🎯 Controlador Genérico Reutilizable para Módulos con Permisos
///
/// Replica la funcionalidad de `modulo_simulado_action` de Python
///
/// # Parámetros
/// - `tmpl`: Motor de plantillas Tera
/// - `pool`: Pool de conexiones a la BD
/// - `breadcrumbs`: Texto del breadcrumb (ej: "Seguridad / Perfiles")
/// - `seccion`: Carpeta de la vista (ej: "seguridad")
/// - `nombre_modulo`: Nombre exacto del módulo en BD (ej: "Perfiles")
pub async fn modulo_generico_action(
    tmpl: web::Data<Tera>,
    pool: web::Data<PgPool>,
    req: HttpRequest,
    breadcrumbs: &str,
    seccion: &str,
    nombre_modulo: &str
) -> HttpResponse {
    // 🔥 Obtener usuario autenticado desde las extensiones
    let user = match req.extensions().get::<UserPayload>() {
        Some(u) => u.clone(),
        None => {
            return HttpResponse::Found().append_header(("Location", "/login")).finish();
        }
    };

    // 2. Obtener TODOS los permisos del perfil
    let todos_los_permisos = permisos_service
        ::get_permisos_by_perfil(&pool, user.id_perfil).await
        .unwrap_or_default();

    println!("\n[DEBUG RUST] Buscando permisos para el módulo: '{}'", nombre_modulo);

    // 3. Buscar el permiso específico del módulo por NOMBRE (igual que Python)
    let permisos_modulo = todos_los_permisos
        .into_iter()
        .find(|p| {
            if let Some(nombre) = &p.strnombremodulo {
                nombre.trim().eq_ignore_ascii_case(nombre_modulo.trim())
            } else {
                false
            }
        })
        .unwrap_or_else(|| {
            println!("[DEBUG RUST] ❌ No se encontró el módulo '{}'. Asignando False por defecto.", nombre_modulo);
            PermisosPerfil {
                idmodulo: None,
                id: None,
                idperfil: Some(user.id_perfil),
                strnombremodulo: Some(nombre_modulo.to_string()),
                bitagregar: false,
                biteditar: false,
                biteliminar: false,
                bitconsulta: false,
                bitdetalle: false,
            }
        });

    println!("[DEBUG RUST] ✅ Permisos encontrados en BD: {:?}", permisos_modulo);

    // 4. Validar permiso de consulta
    if !permisos_modulo.bitconsulta {
        println!("[DEBUG RUST] ⛔ Sin permiso de consulta para '{}'", nombre_modulo);
        return HttpResponse::NotFound().body("No tienes permisos para consultar este módulo");
    }

    // 5. Datos estáticos simulados (igual que Python)
    let datos_estaticos =
        json!([
        {"id": 1, "nombre": format!("Registro Prueba A - {}", nombre_modulo), "estado": "Activo"},
        {"id": 2, "nombre": format!("Registro Prueba B - {}", nombre_modulo), "estado": "Inactivo"},
        {"id": 3, "nombre": format!("Registro Prueba C - {}", nombre_modulo), "estado": "Activo"},
    ]);

    // 6. Obtener el menú dinámico
    let datos_menu = home_service
        ::get_sidebar_menu(&pool, user.id_perfil).await
        .unwrap_or_default();

    // 7. Construir el nombre del archivo de vista (igual que Python)
    let archivo_nombre = nombre_modulo.replace("Principal ", "p").replace(".", "_").to_lowercase();

    let template_path = format!("{}/{}.html", seccion, archivo_nombre);

    let permisos_json = serde_json::to_string(&permisos_modulo).unwrap();
    println!("[DEBUG RUST] JSON que se enviará a la vista: {}", permisos_json);

    // 8. Renderizar la vista hija
    let mut ctx = Context::new();
    ctx.insert("titulo", nombre_modulo);
    ctx.insert("permisos", &permisos_modulo);
    ctx.insert("permisos_json", &permisos_json);
    ctx.insert("datos_json", &serde_json::to_string(&datos_estaticos).unwrap());

    let contenido = tmpl
        .render(&template_path, &ctx)
        .unwrap_or_else(|e| format!("Error en vista '{}': {}", template_path, e));

    // 9. Armar el Layout Final (igual que Python)
    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", nombre_modulo);
    layout_ctx.insert("user_nombre", &user.nombre_completo);
    layout_ctx.insert("user_iniciales", &user.iniciales);
    layout_ctx.insert("user_email", &user.email);
    layout_ctx.insert("breadcrumbs_placeholder", breadcrumbs);
    layout_ctx.insert("datos_menu", &datos_menu);
    layout_ctx.insert("content", &contenido);

    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);

    HttpResponse::Ok().content_type("text/html").body(render)
}
