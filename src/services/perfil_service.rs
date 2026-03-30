use sqlx::PgPool;
use crate::models::perfil::Perfil;

// 🔍 Obtener todos
pub async fn get_all(pool: &PgPool) -> Result<Vec<Perfil>, sqlx::Error> {
    let perfiles = sqlx::query_as::<_, Perfil>(
        r#"
        SELECT id, strnombreperfil, bitadministrador
        FROM perfil
        ORDER BY id DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(perfiles)
}

// 🔍 Obtener por ID
pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Perfil>, sqlx::Error> {
    let perfil = sqlx::query_as::<_, Perfil>(
        r#"
        SELECT id, strnombreperfil, bitadministrador
        FROM perfil
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(perfil)
}

// 💾 Guardar (insert / update)
pub async fn save(
    pool: &PgPool,
    id: Option<i32>,
    nombre: String,
    admin: bool,
) -> Result<String, sqlx::Error> {

    if let Some(id_val) = id {
        sqlx::query(
            "UPDATE perfil SET strnombreperfil = $1, bitadministrador = $2 WHERE id = $3"
        )
        .bind(nombre)
        .bind(admin)
        .bind(id_val)
        .execute(pool)
        .await?;

        Ok("Perfil actualizado".to_string())
    } else {
        sqlx::query(
            "INSERT INTO perfil (strnombreperfil, bitadministrador) VALUES ($1, $2)"
        )
        .bind(nombre)
        .bind(admin)
        .execute(pool)
        .await?;

        Ok("Perfil registrado".to_string())
    }
}

// ❌ Eliminar
pub async fn delete(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
    sqlx::query("DELETE FROM perfil WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok("Perfil eliminado".to_string())
}