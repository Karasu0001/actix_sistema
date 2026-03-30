use std::env;
use sqlx::PgPool;
use sha2::{Sha256, Digest};
use hex;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::models::usuario::{Usuario, UserPayload};


#[derive(Debug, Serialize, Deserialize)]
struct RecaptchaResponse {
    success: bool,
    #[serde(default)]
    challenge_ts: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

pub struct LoginService;

impl LoginService {
    pub async fn verify_recaptcha(token: &str) -> bool {
        println!("[RECAPTCHA] 🔍 Verificando token...");

        // 🔥 Leemos la clave desde el .env
        let recaptcha_secret = env::var("RECAPTCHA_SECRET")
            .expect("Falta la variable RECAPTCHA_SECRET en el archivo .env");
        
        let client = reqwest::Client::new();
        let params = [
            ("secret", recaptcha_secret.as_str()),
            ("response", token),
        ];

        match client
            .post("https://www.google.com/recaptcha/api/siteverify")
            .form(&params)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                match response.json::<RecaptchaResponse>().await {
                    Ok(result) => {
                        if result.success {
                            println!("[RECAPTCHA] ✅ Verificación exitosa");
                            true
                        } else {
                            println!("[RECAPTCHA] ❌ Verificación fallida: {:?}", result.error_codes);
                            false
                        }
                    }
                    Err(e) => {
                        println!("[RECAPTCHA] ❌ Error parseando respuesta: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                println!("[RECAPTCHA] ❌ Error en la petición: {}", e);
                false
            }
        }
    }

    pub async fn authenticate_user(
        pool: &PgPool,
        email: &str,
        password: &str,
    ) -> Result<(bool, String, Option<String>, Option<UserPayload>), String> {
        println!("\n[LOGIN] ➡️ Iniciando intento de autenticación para: '{}'", email);

        // Hash de la contraseña
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let password_hash = hex::encode(hasher.finalize());
        
        println!("[LOGIN] 🔑 Hash generado (primeros 15 chars): {}...", &password_hash[..15]);
        println!("[LOGIN] ⏳ Consultando la base de datos...");

        // Buscar usuario en BD
        let user_result = sqlx::query_as::<_, Usuario>(
            r#"
            SELECT *
            FROM usuario
            WHERE str_correo = $1 
              AND str_pwd = $2 
              AND id_estado_usuario = 1
            "#
        )
        .bind(email)
        .bind(&password_hash)
        .fetch_optional(pool)
        .await;

        match user_result {
            Ok(Some(user)) => {
                println!("[LOGIN] ✅ ¡Éxito! Usuario encontrado en la BD. ID: {}", user.id);

                // Crear payload del JWT
                let exp = (Utc::now() + Duration::hours(8)).timestamp();
                let payload = user.to_payload(exp);

                println!("[LOGIN] 📦 Payload preparado para JWT: Perfil {}", payload.id_perfil);

                // 🔥 Leemos la clave secreta desde el .env
                let jwt_secret = env::var("JWT_SECRET")
                    .expect("Falta la variable JWT_SECRET en el archivo .env");
                

                // Generar token JWT
                match encode(
                    &Header::default(),
                    &payload,
                    &EncodingKey::from_secret(jwt_secret.as_bytes()),
                ) {
                    Ok(token) => {
                        println!("[LOGIN] 🎟️ Token JWT generado correctamente. Retornando éxito.");
                        Ok((true, "Login exitoso".to_string(), Some(token), Some(payload)))
                    }
                    Err(e) => {
                        println!("[LOGIN] ❌ Error generando JWT: {}", e);
                        Ok((false, "Error generando token".to_string(), None, None))
                    }
                }
            }
            Ok(None) => {
                println!("[LOGIN] ❌ Fallo de autenticación. Credenciales incorrectas para '{}'.", email);
                Ok((
                    false,
                    "Correo o contraseña incorrectos, o usuario inactivo".to_string(),
                    None,
                    None,
                ))
            }
            Err(e) => {
                println!("🔥 [LOGIN] ERROR CRÍTICO en authenticate_user: {}", e);
                Err(format!("Error interno del servidor: {}", e))
            }
        }
    }

    /// Decodificar y validar token JWT
    pub fn decode_token(token: &str) -> Option<UserPayload> {

        // 🔥 Leemos la clave secreta desde el .env
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();

        decode::<UserPayload>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .ok()
        .map(|data| data.claims)
    }

    /// Obtener usuario desde la cookie (middleware)
    pub fn get_current_user_from_request(req: &actix_web::HttpRequest) -> Option<UserPayload> {
        if let Some(cookie) = req.cookie("auth_token") {
            Self::decode_token(cookie.value())
        } else {
            None
        }
    }
}