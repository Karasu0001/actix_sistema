use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SubModuloDTO {
    pub ruta: String,
    pub nombre: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuSidebarDTO {
    pub id_html: String,
    pub titulo: String,
    pub submodulos: Vec<SubModuloDTO>,
}

// Estructura auxiliar para la consulta plana a la BD
#[derive(sqlx::FromRow)]
struct MenuFilaDB {
    id_menu: i32,
    nombre_menu: String,
    ruta_modulo: Option<String>,
    nombre_modulo: String,
}

pub async fn get_sidebar_menu(pool: &PgPool, id_perfil: i32) -> Result<Vec<MenuSidebarDTO>, sqlx::Error> {
    let es_admin = sqlx::query_scalar::<_, bool>(
        "SELECT bitAdministrador FROM Perfil WHERE id = $1"
    )
    .bind(id_perfil)
    .fetch_optional(pool)
    .await?
    .unwrap_or(false);

    let filas = if es_admin {
        // 🟢 Usamos query_as (sin signo de exclamación) y le pasamos el tipo
        sqlx::query_as::<_, MenuFilaDB>(
            r#"
            SELECT me.id AS id_menu, me.strNombreMenu AS nombre_menu,
                   mo.strRuta AS ruta_modulo, mo.strNombreModulo AS nombre_modulo
            FROM Menu me
            JOIN Modulo mo ON me.id = mo.idMenu
            ORDER BY me.id, mo.id
            "#
        )
        .fetch_all(pool)
        .await?
    } else {
        // 🟢 Usamos query_as (sin signo de exclamación) y bindeamos la variable manualmente
        sqlx::query_as::<_, MenuFilaDB>(
            r#"
            SELECT me.id AS id_menu, me.strNombreMenu AS nombre_menu,
                   mo.strRuta AS ruta_modulo, mo.strNombreModulo AS nombre_modulo
            FROM Menu me
            JOIN Modulo mo ON me.id = mo.idMenu
            JOIN PermisosPerfil pp ON mo.id = pp.id_modulo
            WHERE pp.id_perfil = $1 AND pp.bit_consulta = true
            ORDER BY me.id, mo.id
            "#
        )
        .bind(id_perfil) // 🟢 IMPORTANTE: Bind manual al quitar el macro
        .fetch_all(pool)
        .await?
    };

    // 3. Agrupar los resultados en la estructura Jerárquica (Padre -> Hijos)
    let mut menu_agrupado: Vec<MenuSidebarDTO> = Vec::new();
    
    for fila in filas {
        // Buscamos si el menú padre ya existe en nuestro vector
        if let Some(menu) = menu_agrupado.iter_mut().find(|m| m.titulo == fila.nombre_menu) {
            menu.submodulos.push(SubModuloDTO {
                ruta: fila.ruta_modulo.unwrap_or_default(),
                nombre: fila.nombre_modulo,
            });
        } else {
            // Si no existe, lo creamos
            menu_agrupado.push(MenuSidebarDTO {
                id_html: format!("menu_{}", fila.id_menu), // ej. "menu_1"
                titulo: fila.nombre_menu,
                submodulos: vec![SubModuloDTO {
                    ruta: fila.ruta_modulo.unwrap_or_default(),
                    nombre: fila.nombre_modulo,
                }],
            });
        }
    }

    Ok(menu_agrupado)
}