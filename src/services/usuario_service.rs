use sqlx::PgPool;
use crate::models::usuario::Usuario;
use sha2::{ Sha256, Digest };

pub struct UsuarioService;

impl UsuarioService {
    pub async fn get_all_users(pool: &PgPool) -> Vec<Usuario> {
        println!("\n--- INICIANDO GET_ALL_USERS (NEON DB) ---");

        let result: Result<Vec<Usuario>, sqlx::Error> = sqlx
            ::query_as::<_, Usuario>(
                r#"SELECT id, usuario, email, password, 
               TO_CHAR(created_at, 'YYYY-MM-DD HH24:MI:SS') as created_at 
               FROM usuarios ORDER BY id DESC"#
            )
            .fetch_all(pool).await;

        match result {
            Ok(users) => {
                println!(" DEBUG: Usuarios encontrados: {}", users.len());
                users
            }
            Err(e) => {
                // Este print te dirá exactamente si hay otro error de mapeo
                println!(" ERROR en get_all_users: {}", e);
                vec![]
            }
        }
    }

    pub async fn register_user(pool: &PgPool, mut user: Usuario) -> (bool, String) {
        // Corregimos la inferencia del hasher
        if let Some(ref pwd) = user.password {
            let mut hasher = Sha256::new();
            let bytes: &[u8] = pwd.as_bytes(); // Forzamos el tipo a slice de bytes
            hasher.update(bytes);
            user.password = Some(format!("{:x}", hasher.finalize()));
        }

        let query =
            r#"
            INSERT INTO usuarios (usuario, email, password, created_at) 
            VALUES ($1, $2, $3, NOW())
        "#;

        let result = sqlx
            ::query(query)
            .bind(&user.usuario)
            .bind(&user.email)
            .bind(&user.password)
            .execute(pool).await;

        match result {
            Ok(_) => (true, "Usuario registrado exitosamente".to_string()),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("usuarios_email_key") || error_msg.contains("23505") {
                    return (false, "Este correo electrónico ya está registrado".to_string());
                }
                (false, format!("Error en base de datos: {}", error_msg))
            }
        }
    }

    pub async fn get_user_by_id(pool: &PgPool, user_id: i32) -> Option<Usuario> {
        let result: Result<Option<Usuario>, sqlx::Error> = sqlx
            ::query_as::<_, Usuario>(
                r#"SELECT id, usuario, email, password, 
               TO_CHAR(created_at, 'YYYY-MM-DD HH24:MI:SS') as created_at 
               FROM usuarios WHERE id = $1"#
            )
            .bind(user_id)
            .fetch_optional(pool).await;

        match result {
            Ok(u) => u,
            Err(e) => {
                println!(" ERROR en get_user_by_id: {}", e);
                None
            }
        }
    }

    pub async fn update_existing_user(pool: &PgPool, user: Usuario) -> (bool, String) {
        let query = "UPDATE usuarios SET usuario=$1, email=$2 WHERE id=$3";

        let result = sqlx
            ::query(query)
            .bind(&user.usuario)
            .bind(&user.email)
            .bind(user.id)
            .execute(pool).await;

        match result {
            Ok(_) => (true, "Usuario actualizado correctamente".to_string()),
            Err(e) => (false, e.to_string()),
        }
    }

    pub async fn delete_user(pool: &PgPool, user_id: i32) -> (bool, String) {
        let result = sqlx
            ::query("DELETE FROM usuarios WHERE id = $1")
            .bind(user_id)
            .execute(pool).await;

        match result {
            Ok(_) => (true, "Usuario eliminado correctamente".to_string()),
            Err(e) => (false, e.to_string()),
        }
    }
}
