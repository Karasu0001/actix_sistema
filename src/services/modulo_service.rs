use sqlx::PgPool;
use crate::models::modulo::{ModuloRow, MenuRow};

// 🔍 Obtener todos
pub async fn get_all(pool: &PgPool) -> Result<Vec<ModuloRow>, sqlx::Error> {
    let modulos = sqlx::query_as::<_, ModuloRow>(
        r#"
        SELECT mo.id, mo.strnombremodulo, mo.idmenu, me.strnombremenu, mo.strruta 
        FROM modulo mo
        INNER JOIN menu me ON mo.idmenu = me.id
        ORDER BY mo.id DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(modulos)
}

// 🔍 Obtener por ID
pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<ModuloRow>, sqlx::Error> {
    let modulo = sqlx::query_as::<_, ModuloRow>(
        r#"
        SELECT mo.id, mo.strnombremodulo, mo.idmenu, me.strnombremenu, mo.strruta 
        FROM modulo mo
        INNER JOIN menu me ON mo.idmenu = me.id
        WHERE mo.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(modulo)
}

// 💾 Guardar (Transacción: Busca/Crea Menú y luego Guarda Módulo)
pub async fn save(
    pool: &PgPool,
    id: Option<i32>,
    nombre_modulo: String,
    nombre_menu: String, // El texto ingresado por el usuario
    ruta: Option<String>,
) -> Result<String, sqlx::Error> {

    // Iniciamos una transacción
    let mut tx = pool.begin().await?;

    // 📦 Struct temporal solo para atrapar el ID que nos devuelven las consultas
    #[derive(sqlx::FromRow)]
    struct RecordId {
        id: i32,
    }

    // 1. Buscar si el Menú ya existe (cambiamos query! por query_as)
    let menu_existente = sqlx::query_as::<_, RecordId>("SELECT id FROM menu WHERE strnombremenu = $1")
        .bind(&nombre_menu)
        .fetch_optional(&mut *tx)
        .await?;

    let id_menu = match menu_existente {
        Some(record) => record.id, // Si existe, tomamos el ID
        None => {
            // 2. Si no existe, lo insertamos y obtenemos el nuevo ID
            let nuevo_menu = sqlx::query_as::<_, RecordId>(
                "INSERT INTO menu (strnombremenu) VALUES ($1) RETURNING id"
            )
            .bind(&nombre_menu)
            .fetch_one(&mut *tx)
            .await?;
            
            nuevo_menu.id
        }
    };

    // 3. Guardar el Módulo
    if let Some(id_val) = id { // UPDATE
        sqlx::query("UPDATE modulo SET strnombremodulo = $1, idmenu = $2, strruta = $3 WHERE id = $4")
            .bind(nombre_modulo)
            .bind(id_menu)
            .bind(ruta)
            .bind(id_val)
            .execute(&mut *tx)
            .await?;
        
        tx.commit().await?;
        Ok("Módulo actualizado".to_string())
    } else { // INSERT
        sqlx::query("INSERT INTO modulo (strnombremodulo, idmenu, strruta) VALUES ($1, $2, $3)")
            .bind(nombre_modulo)
            .bind(id_menu)
            .bind(ruta)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok("Módulo registrado".to_string())
    }
}

// ❌ Eliminar Módulo
pub async fn delete(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
    sqlx::query("DELETE FROM modulo WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok("Módulo eliminado".to_string())
}

// ==========================================
// 🧩 SERVICIOS EXTRAS PARA LOS MENÚS
// ==========================================

pub async fn get_menus(pool: &PgPool) -> Result<Vec<MenuRow>, sqlx::Error> {
    let menus = sqlx::query_as::<_, MenuRow>("SELECT id, strnombremenu FROM menu ORDER BY strnombremenu ASC")
        .fetch_all(pool)
        .await?;
    Ok(menus)
}

pub async fn update_menu(pool: &PgPool, id: i32, nombre: String) -> Result<String, sqlx::Error> {
    sqlx::query("UPDATE menu SET strnombremenu = $1 WHERE id = $2")
        .bind(nombre)
        .bind(id)
        .execute(pool)
        .await?;
    Ok("Menú actualizado correctamente".to_string())
}

pub async fn delete_menu(pool: &PgPool, id: i32) -> Result<String, sqlx::Error> {
    sqlx::query("DELETE FROM menu WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok("Menú eliminado correctamente".to_string())
}