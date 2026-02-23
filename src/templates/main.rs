use actix_files as fs;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::io::Write;
use uuid::Uuid;

// Modelos

#[derive(Deserialize)]
struct UsuarioForm {
    usuario: String,
    email: String,
    password: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct CarruselItem {
    id: i32,
    nombre_archivo: String,
    ruta_relativa: String,
}

//Vistas

async fn pantalla1() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("templates/pantalla1.html"))
}

async fn mantenimiento() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("templates/mantenimiento.html"))
}

async fn pantalla2() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("templates/pantalla2.html"))
}

async fn pantalla3() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("templates/pantalla3.html"))
}

async fn pagina_404() -> impl Responder {
    HttpResponse::NotFound()
        .content_type("text/html")
        .body(include_str!("templates/404.html"))
}

//Carrusel

async fn carrusel_view(pool: web::Data<PgPool>) -> impl Responder {
    // Obtener imágenes de la DB
    let items = sqlx::query_as::<_, CarruselItem>(
        "SELECT id, nombre_archivo, ruta_relativa FROM carrusel ORDER BY fecha_carga DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let mut items_html = String::new();

    if items.is_empty() {
        items_html = r#"<div class="carousel-item active"><img src="https://via.placeholder.com/1200x500?text=No+hay+imagenes" class="d-block w-100"></div>"#.to_string();
    } else {
        for (i, item) in items.iter().enumerate() {
            let active = if i == 0 { "active" } else { "" };
            items_html.push_str(&format!(
                r#"<div class="carousel-item {}">
                    <img src="{}" class="d-block w-100" style="height: 500px; object-fit: cover;" alt="Imagen">
                </div>"#, active, item.ruta_relativa
            ));
        }
    }

    // Reemplazamos el placeholder en el HTML
    let html_base = include_str!("templates/carrusel.html");
    let html_final = html_base.replace("{{items_carrusel}}", &items_html);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_final)
}

async fn subir_imagen(pool: web::Data<PgPool>, mut payload: Multipart) -> impl Responder {
    let upload_dir = "./static/Images/Carrusel";
    let mut filename = String::new();

    // Crear carpeta si no existe
    let _ = std::fs::create_dir_all(upload_dir);

    // Procesar el archivo enviado
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let original_name = content_disposition.get_filename().unwrap_or("image.png");
        let extension = std::path::Path::new(original_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");

        filename = format!("{}.{}", Uuid::new_v4(), extension);
        let filepath = format!("{}/{}", upload_dir, filename);

        let mut f = match std::fs::File::create(&filepath) {
            Ok(file) => file,
            Err(_) => return HttpResponse::InternalServerError().body("Error creando archivo"),
        };

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).unwrap();
        }
    }

    let web_path = format!("/static/Images/Carrusel/{}", filename);

    // Guardar en Postgres
    let result =
        sqlx::query("INSERT INTO carrusel (nombre_archivo, ruta_relativa) VALUES ($1, $2)")
            .bind(&filename)
            .bind(&web_path)
            .execute(pool.get_ref())
            .await;

    match result {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/carrusel"))
            .finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error DB: {}", e)),
    }
}

// Usuarios

async fn crear_usuario(pool: web::Data<PgPool>, form: web::Form<UsuarioForm>) -> impl Responder {
    let resultado =
        sqlx::query("INSERT INTO usuarios (usuario, email, password) VALUES ($1, $2, $3)")
            .bind(&form.usuario)
            .bind(&form.email)
            .bind(&form.password)
            .execute(pool.get_ref())
            .await;

    match resultado {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", "/pantalla1"))
            .finish(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Error al registrar: el usuario ya existe")
        }
    }
}

// Main

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL no definida");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error conectando a la base de datos");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT inválido");

    println!("🚀 Servidor en http://localhost:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // 1. Servir archivos estáticos (CSS, JS e Imágenes subidas)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            // 2. Rutas del Carrusel
            .route("/carrusel", web::get().to(carrusel_view))
            .route("/carrusel", web::post().to(subir_imagen))
            // 3. Rutas de Usuario y Vistas
            .route("/", web::get().to(mantenimiento))
            .route("/pantalla1", web::get().to(pantalla1))
            .route("/mantenimiento", web::get().to(mantenimiento))
            .route("/pantalla2", web::get().to(pantalla2))
            .route("/pantalla3", web::get().to(pantalla3))
            .route("/crear_usuario", web::post().to(crear_usuario))
            // 4. 404
            .default_service(web::route().to(pagina_404))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
