use sqlx::PgPool;
use crate::models::catalogo::*;

pub async fn get_sexos(pool: &PgPool) -> Result<Vec<Sexo>, sqlx::Error> {
    sqlx::query_as::<_, Sexo>("SELECT id, strsexo FROM sexo ORDER BY strsexo")
        .fetch_all(pool)
        .await
}

pub async fn get_estados(pool: &PgPool) -> Result<Vec<EstadoUsuario>, sqlx::Error> {
    sqlx::query_as::<_, EstadoUsuario>("SELECT id, strnombreestado FROM estadousuario ORDER BY strnombreestado")
        .fetch_all(pool)
        .await
}

pub async fn get_perfiles(pool: &PgPool) -> Result<Vec<PerfilCatalogo>, sqlx::Error> {
    sqlx::query_as::<_, PerfilCatalogo>("SELECT id, strnombreperfil, bitadministrador FROM perfil ORDER BY strnombreperfil")
        .fetch_all(pool)
        .await
}

pub async fn get_modulos(pool: &PgPool) -> Result<Vec<ModuloCatalogo>, sqlx::Error> {
    sqlx::query_as::<_, ModuloCatalogo>("SELECT id, strnombremodulo, idmenu, strruta FROM modulo ORDER BY strnombremodulo")
        .fetch_all(pool)
        .await
}

pub async fn get_menus(pool: &PgPool) -> Result<Vec<Menu>, sqlx::Error> {
    sqlx::query_as::<_, Menu>("SELECT id, strnombremenu FROM menu ORDER BY strnombremenu")
        .fetch_all(pool)
        .await
}