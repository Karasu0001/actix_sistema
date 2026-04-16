use sqlx::PgPool;
use crate::models::usuario::Usuario;
use crate::controllers::usuario_controller::UsuarioDTO;
use sha2::{Sha256, Digest};
use chrono::NaiveDate;

// Recupera la lista completa de usuarios ordenados por ID descendente.
pub async fn get_all(pool: &PgPool) -> Result<Vec<Usuario>, sqlx::Error> {
    let usuarios = sqlx::query_as::<_, Usuario>(
        "SELECT * FROM usuario ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(usuarios)
}

// Busca un usuario específico mediante su identificador principal.
pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Usuario>, sqlx::Error> {
    let usuario = sqlx::query_as::<_, Usuario>(
        "SELECT * FROM usuario WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    
    Ok(usuario)
}

// Gestiona tanto la creación como la actualización de usuarios dependiendo de si se recibe un ID.
pub async fn save(
    pool: &PgPool,
    data: UsuarioDTO,
) -> Result<String, sqlx::Error> {
    
    // Genera el hash de la contraseña solo si el cliente envía un valor válido y no vacío.
    let pwd_hash = if let Some(ref pwd) = data.strPwd {
        if !pwd.trim().is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(pwd.as_bytes());
            Some(hex::encode(hasher.finalize()))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(id_val) = data.id {
        // Actualiza los datos. Se utiliza COALESCE para conservar la imagen actual si no se proporciona una nueva.
        sqlx::query(
            "UPDATE usuario SET 
                nombre=$1, 
                apellido_p=$2, 
                apellido_m=$3, 
                id_perfil=$4, 
                fecha_nacimiento=$5, 
                id_estado_usuario=$6, 
                id_sexo=$7, 
                str_correo=$8, 
                str_numero_celular=$9,
                str_imagen_path=COALESCE($10, str_imagen_path),
                str_pwd=COALESCE($11, str_pwd)
             WHERE id=$12"
        )
        .bind(&data.nombre)
        .bind(&data.apellidoP)
        .bind(&data.apellidoM)
        .bind(data.idPerfil)
        .bind(data.fechaNacimiento)
        .bind(data.idEstadoUsuario)
        .bind(data.idSexo)
        .bind(&data.strCorreo)
        .bind(&data.strNumeroCelular)
        .bind(&data.strImagenPath)
        .bind(pwd_hash)
        .bind(id_val)
        .execute(pool)
        .await?;

        Ok("Usuario actualizado exitosamente".to_string())
    } else {
        // Registra un nuevo usuario en el sistema.
        sqlx::query(
            r#"INSERT INTO usuario 
            (nombre, apellido_p, apellido_m, id_perfil, str_pwd, fecha_nacimiento, id_estado_usuario, id_sexo, str_correo, str_numero_celular, str_imagen_path, fecha_registro) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)"#
        )
        .bind(&data.nombre)
        .bind(&data.apellidoP)
        .bind(&data.apellidoM)
        .bind(data.idPerfil)
        .bind(pwd_hash)
        .bind(data.fechaNacimiento)
        .bind(data.idEstadoUsuario.unwrap_or(1))
        .bind(data.idSexo)
        .bind(&data.strCorreo)
        .bind(&data.strNumeroCelular)
        .bind(&data.strImagenPath) 
        .execute(pool)
        .await?;

        Ok("Usuario registrado exitosamente".to_string())
    }
}

// Elimina el registro del usuario de la base de datos de forma permanente.
pub async fn delete(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
    sqlx::query("DELETE FROM usuario WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
        
    Ok("Usuario eliminado exitosamente".to_string())
}
