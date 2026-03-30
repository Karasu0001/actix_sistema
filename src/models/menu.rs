use serde::{Deserialize, Serialize}; // <-- Importante

#[derive(Serialize, Deserialize, Debug, Clone)] // <-- Fundamental para Tera
pub struct MenuEstructurado {
    pub id_html: String,
    pub titulo: String,
    pub submodulos: Vec<SubModulo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubModulo {
    pub ruta: String,
    pub nombre: String,
}

// Estructura plana para atrapar la consulta SQL
#[derive(sqlx::FromRow)]
pub struct MenuRow {
    pub id_menu: i32,
    pub nombre_menu: String,
    pub id_modulo: i32,
    pub nombre_modulo: String,
    pub ruta: Option<String>,
}
