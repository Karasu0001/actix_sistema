use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use tera::{Tera, Context};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::NaiveDate;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::io::Write;

use crate::services::usuario_service;
use crate::services::{permisos_service, home_service};
use crate::models::permiso::PermisosPerfil;
use crate::models::usuario::UserPayload;

// 📦 DTO para recibir datos del Frontend
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioDTO {
    pub id: Option<i32>,
    pub nombre: String,
    pub apellidoP: Option<String>,
    pub apellidoM: Option<String>,
    pub strCorreo: Option<String>,
    pub strPwd: Option<String>,
    pub fechaNacimiento: Option<NaiveDate>,
    pub idPerfil: Option<i32>,
    pub idEstadoUsuario: Option<i32>,
    pub idSexo: Option<i32>,
    pub strNumeroCelular: Option<String>,
    pub strImagenPath: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermisosModuloDTO {
    pub str_nombre_modulo: String,
    pub bit_agregar: bool,
    pub bit_editar: bool,
    pub bit_eliminar: bool,
    pub bit_consulta: bool,
}

// 📄 Vista Renderizada (Usuarios)
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
    
    let todos_los_permisos = permisos_service::get_permisos_by_perfil(&pool, user.id_perfil)
        .await
        .unwrap_or_default();

    let permisos_usuario = todos_los_permisos.into_iter()
        .find(|p| p.idmodulo == Some(4)) 
        .unwrap_or_else(|| {
            PermisosPerfil {
                idmodulo: Some(4), id: None, idperfil: Some(user.id_perfil), strnombremodulo: None,
                bitagregar: false, biteditar: false, biteliminar: false, bitconsulta: false, bitdetalle: false
            }
        });

    if !permisos_usuario.bitconsulta {
        return HttpResponse::NotFound().body("No tienes permisos para consultar este módulo");
    }

    // 🔥 OBTENEMOS EL MENÚ (Asegúrate de que las structs en home_service tengan #[derive(Serialize)])
    let datos_menu = home_service::get_sidebar_menu(&pool, user.id_perfil).await.unwrap_or_default();
    
     // 🔍 DEBUG TEMPORAL
    println!("📋 Menú obtenido: {:?}", datos_menu);
    println!("📊 Cantidad de items: {}", datos_menu.len());
    
    let mut ctx = Context::new();
    ctx.insert("permisos", &permisos_usuario);
    ctx.insert("permisos_json", &serde_json::to_string(&permisos_usuario).unwrap());

    // Renderizamos el contenido interno primero
    let contenido = tmpl.render("seguridad/usuarios.html", &ctx).unwrap_or_else(|e| format!("Error en vista: {}", e));

    let mut layout_ctx = Context::new();
    layout_ctx.insert("titulo", "Gestión de Usuarios");
  layout_ctx.insert("user_nombre", &user.nombre_completo); // 👈 Dinámico
    layout_ctx.insert("user_iniciales", &user.iniciales);    // 👈 Dinámico
    layout_ctx.insert("user_email", &user.email);        // 👈 Dinámico
    layout_ctx.insert("breadcrumbs_placeholder", "Seguridad / Usuario");
    
    // 🔥 PASAMOS LA VARIABLE DEL MENÚ AL CONTEXTO DEL LAYOUT
    layout_ctx.insert("datos_menu", &datos_menu); 
    
    layout_ctx.insert("content", &contenido);

    // Renderizamos usando nuestra función detallada (¡Asegúrate de haber actualizado src/render.rs!)
    let render = crate::render::render_view(&tmpl, "home/layout.html", layout_ctx);
    
    HttpResponse::Ok().content_type("text/html").body(render)
}

// 📡 API: GET ALL
pub async fn get_all(pool: web::Data<PgPool>) -> HttpResponse {
    match usuario_service::get_all(&pool).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "msg": e.to_string()})),
    }
}

// 📡 API: GET BY ID
pub async fn get_by_id(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    match usuario_service::get_by_id(&pool, path.into_inner()).await {
        Ok(Some(data)) => HttpResponse::Ok().json(data),
        Ok(None) => HttpResponse::NotFound().json(json!({"success": false, "msg": "No encontrado"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "msg": e.to_string()})),
    }
}

// 📡 API: SAVE con soporte para archivos 
pub async fn save(
    pool: web::Data<PgPool>,
    mut payload: Multipart,
) -> HttpResponse {
    let mut data = UsuarioDTO {
        id: None,
        nombre: String::new(),
        apellidoP: None,
        apellidoM: None,
        strCorreo: None,
        strPwd: None,
        fechaNacimiento: None,
        idPerfil: None,
        idEstadoUsuario: None,
        idSexo: None,
        strNumeroCelular: None,
        strImagenPath: None,
    };

    // Procesar cada campo del multipart
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => {
                return HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "msg": format!("Error al procesar campo: {}", e)
                }));
            }
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("").to_string();
        let filename = content_disposition.get_filename().map(|s| s.to_string());

        match field_name.as_str() {
            "id" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                if let Ok(s) = String::from_utf8(bytes.to_vec()) { data.id = s.parse::<i32>().ok(); }
            }
            "nombre" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                data.nombre = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            }
            "apellidoP" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                data.apellidoP = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            }
            "apellidoM" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                data.apellidoM = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            }
            "strCorreo" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                data.strCorreo = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            }
            "strPwd" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                let pwd = String::from_utf8(bytes.to_vec()).unwrap_or_default();
                if !pwd.is_empty() { data.strPwd = Some(pwd); }
            }
            "fechaNacimiento" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                if let Ok(s) = String::from_utf8(bytes.to_vec()) { data.fechaNacimiento = NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok(); }
            }
            "idSexo" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                if let Ok(s) = String::from_utf8(bytes.to_vec()) { data.idSexo = s.parse::<i32>().ok(); }
            }
            "idPerfil" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                if let Ok(s) = String::from_utf8(bytes.to_vec()) { data.idPerfil = s.parse::<i32>().ok(); }
            }
            "idEstadoUsuario" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                if let Ok(s) = String::from_utf8(bytes.to_vec()) { data.idEstadoUsuario = s.parse::<i32>().ok(); }
            }
            "strNumeroCelular" => {
                let mut bytes = web::BytesMut::new();
                while let Some(chunk) = field.next().await { bytes.extend_from_slice(&chunk.unwrap()); }
                data.strNumeroCelular = Some(String::from_utf8(bytes.to_vec()).unwrap_or_default());
            }
           "imagenInput" => {
                if let Some(fname) = filename {
                    // Evitar procesar si el navegador manda un archivo vacío sin nombre
                    if !fname.trim().is_empty() {
                        // 1. Aseguramos que la carpeta exista antes de guardar
                        std::fs::create_dir_all("./static/images/usuarios").ok();
                        
                        // 2. Quitamos espacios del nombre para evitar problemas web
                        let clean_fname = fname.replace(" ", "_");
                        
                        // 3. Ruta de guardado físico en el servidor
                        let filepath = format!("./static/images/usuarios/{}", clean_fname);
                        let filepath_clone = filepath.clone();
                        
                        let mut f = match web::block(move || std::fs::File::create(&filepath_clone)).await {
                            Ok(Ok(file)) => file,
                            _ => continue,
                        };

                        while let Some(chunk) = field.next().await {
                            let data_chunk = chunk.unwrap();
                            let write_result = web::block(move || {
                                use std::io::Write; // Asegúrate de tener esto o importarlo arriba
                                f.write_all(&data_chunk).map(|_| f)
                            }).await;
                            
                            match write_result {
                                Ok(Ok(file)) => f = file,
                                _ => break,
                            }
                        }
                        
                        // 4. Ruta relativa que guardaremos en la Base de Datos
                        data.strImagenPath = Some(format!("static/images/usuarios/{}", clean_fname));
                    }
                }
            }
            _ => {}
        }
    }

    println!("📥 Datos listos para guardar: {:?}", data);

    // AQUÍ llamamos al servicio de la base de datos
    match usuario_service::save(&pool, data).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"success": true, "msg": msg})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "msg": e.to_string()})),
    }
}

// 📡 API: DELETE
pub async fn delete(pool: web::Data<PgPool>, path: web::Path<i32>) -> HttpResponse {
    match usuario_service::delete(&pool, path.into_inner()).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"success": true, "msg": msg})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "msg": e.to_string()})),
    }
}