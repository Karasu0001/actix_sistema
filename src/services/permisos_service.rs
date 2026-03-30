use sqlx::PgPool;
use crate::models::permiso::PermisosPerfil;

/// 🔍 Obtener permisos por Perfil (Lógica para validación de acceso)
pub async fn get_permisos_by_perfil(pool: &PgPool, id_perfil: i32) -> Result<Vec<PermisosPerfil>, sqlx::Error> {
    // 1. Validar si el perfil es Administrador
    let perfil_admin = sqlx::query_scalar::<_, bool>(
        "SELECT bitadministrador FROM perfil WHERE id = $1"
    )
    .bind(id_perfil)
    .fetch_optional(pool)
    .await?;

    if let Some(es_admin) = perfil_admin {
        if es_admin {
            // 2A. Si es admin: todos los permisos en true
            let permisos = sqlx::query_as::<_, PermisosPerfil>(
                r#"
                SELECT 
                    M.id AS idmodulo, 
                    NULL::integer AS id, 
                    $1 AS idperfil,
                    M.strnombremodulo AS strnombremodulo,
                    true AS bitagregar,
                    true AS biteditar,
                    true AS biteliminar,
                    true AS bitconsulta,
                    true AS bitdetalle
                FROM modulo M
                ORDER BY M.strnombremodulo
                "#
            )
            .bind(id_perfil)
            .fetch_all(pool)
            .await?;
            return Ok(permisos);
        }
    } else {
        return Ok(vec![]); // Perfil no existe
    }

    // 2B. Si no es admin: traer permisos reales
    let permisos = sqlx::query_as::<_, PermisosPerfil>(
        r#"
        SELECT 
            M.id AS idmodulo, 
            P.id AS id,
            $1 AS idperfil,
            M.strnombremodulo AS strnombremodulo,
            COALESCE(P.bit_agregar, false) as bitagregar,
            COALESCE(P.bit_editar, false) as biteditar,
            COALESCE(P.bit_eliminar, false) as biteliminar,
            COALESCE(P.bit_consulta, false) as bitconsulta,
            COALESCE(P.bit_detalle, false) as bitdetalle
        FROM modulo M
        LEFT JOIN permisosperfil P ON M.id = P.id_modulo AND P.id_perfil = $1
        ORDER BY M.strnombremodulo
        "#
    )
    .bind(id_perfil)
    .fetch_all(pool)
    .await?;

    Ok(permisos)
}

/// 👁️ Obtener permisos para la Vista de Gestión
pub async fn get_permisos_by_view_perfil(pool: &PgPool, id_perfil: i32) -> Result<Vec<PermisosPerfil>, sqlx::Error> {
    let permisos = sqlx::query_as::<_, PermisosPerfil>(
        r#"
        SELECT 
            M.id AS idmodulo, 
            P.id AS id,
            $1 AS idperfil,
            COALESCE(P.bit_agregar, false) as bitagregar,
            COALESCE(P.bit_editar, false) as biteditar,
            COALESCE(P.bit_eliminar, false) as biteliminar,
            COALESCE(P.bit_consulta, false) as bitconsulta,
            COALESCE(P.bit_detalle, false) as bitdetalle
        FROM modulo M
        LEFT JOIN PermisosPerfil P ON M.id = P.id_modulo AND P.id_perfil = $1
        "#
    )
    .bind(id_perfil)
    .fetch_all(pool)
    .await?;

    Ok(permisos)
}

/// 💾 Guardar o Actualizar Permiso (Upsert)
pub async fn update_permiso(
    pool: &PgPool,
    data: PermisosPerfil
) -> Result<String, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO PermisosPerfil (id_modulo, id_perfil, bit_agregar, bit_editar, bit_eliminar, bit_consulta, bit_detalle)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id_modulo, id_perfil) 
        DO UPDATE SET 
            bit_agregar = EXCLUDED.bit_agregar,
            bit_editar = EXCLUDED.bit_editar,
            bit_eliminar = EXCLUDED.bit_eliminar,
            bit_consulta = EXCLUDED.bit_consulta,
            bit_detalle = EXCLUDED.bit_detalle
        "#
    )
    .bind(data.idmodulo)
    .bind(data.idperfil)
    .bind(data.bitagregar)
    .bind(data.biteditar)
    .bind(data.biteliminar)
    .bind(data.bitconsulta)
    .bind(data.bitdetalle)
    .execute(pool)
    .await?;

    Ok("Permiso actualizado con éxito".to_string())
}