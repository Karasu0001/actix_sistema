use actix_web::web;
use crate::controllers::home_controller;
use crate::controllers::perfil_controller;
use crate::controllers::permisos_controller; 
use crate::controllers::modulo_controller; 
use crate::controllers::catalogo_controller;
use crate::controllers::usuario_controller;
use crate::controllers::modulo_generico_controller;
use crate::controllers::error_controller;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        //  Home
        .route("/", web::get().to(home_controller::index))

        // Login
        .route("/login", web::get().to(home_controller::login_index))
        .route("/logout", web::get().to(home_controller::logout_action))
        .route("/api/login",  web::post().to(home_controller::login_api_dispatcher))

        // ==========================================
        //  PERFILES
        // ==========================================
        //  Vista
        .route("/perfiles", web::get().to(perfil_controller::index))

        //  API
        .route("/api/perfiles", web::get().to(perfil_controller::get_all))
        .route("/api/perfiles/{id}", web::get().to(perfil_controller::get_by_id))
        .route("/api/perfiles", web::post().to(perfil_controller::save))
        .route("/api/perfiles/{id}", web::delete().to(perfil_controller::delete))

        // ==========================================
        //  PERMISOS
        // ==========================================
        //  Vista principal de gestión de permisos
        .route("/permisos", web::get().to(permisos_controller::permisos_manager_index))

        // 📡 API
        .route("/api/permisos_perfil/{id}", web::get().to(permisos_controller::get_permisos_by_perfil))
        .route("/api/permisos_perfil", web::post().to(permisos_controller::bulk_update_permisos))
        
        // ==========================================
        //  MÓDULOS
        // ==========================================
        //  Vista Módulos
        .route("/modulos", web::get().to(modulo_controller::index))

        //  API Módulos
        .route("/api/modulos", web::get().to(modulo_controller::get_all))
        .route("/api/modulos/{id}", web::get().to(modulo_controller::get_by_id))
        .route("/api/modulos", web::post().to(modulo_controller::save))
        .route("/api/modulos/{id}", web::delete().to(modulo_controller::delete))

        // ==========================================
        //  USUARIOS
        // ==========================================
        //  Vista Usuarios
        .route("/usuarios", web::get().to(usuario_controller::index))

        //  API Usuarios
        .route("/api/usuarios", web::get().to(usuario_controller::get_all))
        .route("/api/usuarios/{id}", web::get().to(usuario_controller::get_by_id))
        .route("/api/usuarios", web::post().to(usuario_controller::save))
        .route("/api/usuarios", web::put().to(usuario_controller::save)) 
        .route("/api/usuarios/{id}", web::delete().to(usuario_controller::delete))

        // ==========================================
        //  CATÁLOGOS
        // ==========================================
        .route("/api/perfil", web::get().to(catalogo_controller::get_perfiles))
        .route("/api/sexos", web::get().to(catalogo_controller::get_sexos))
        .route("/api/estados", web::get().to(catalogo_controller::get_estados))
        .route("/api/menus", web::get().to(catalogo_controller::get_menus))

        // ==========================================
        //  MÓDULOS GENÉRICOS SIMULADOS
        // ==========================================
        // Principal 1
        .route("/p1-1", web::get().to(modulo_generico_controller::p1_1_index))
        .route("/p1-2", web::get().to(modulo_generico_controller::p1_2_index))
        
        // Principal 2
        .route("/p2-1", web::get().to(modulo_generico_controller::p2_1_index))
        .route("/p2-2", web::get().to(modulo_generico_controller::p2_2_index)) // ⚠️ ASEGÚRATE DE QUITAR EL PUNTO Y COMA AQUÍ

        // ==========================================
        //  RUTA CATCH-ALL (ERROR 404)
        // ==========================================
        .default_service(web::route().to(error_controller::not_found)); // ✅ EL PUNTO Y COMA VA AQUÍ
}