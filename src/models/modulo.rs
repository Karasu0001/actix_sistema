use serde::{Serialize, Deserialize};
use sqlx::FromRow;

// Modelo para la lectura con el JOIN (Modulo + Menu)
#[derive(Serialize, Deserialize, FromRow)]
pub struct ModuloRow {
    pub id: i32,
    #[serde(rename = "strNombreModulo")]
    pub strnombremodulo: String,
    #[serde(rename = "idMenu")]
    pub idmenu: Option<i32>,
    #[serde(rename = "strNombreMenu")]
    pub strnombremenu: Option<String>,
    #[serde(rename = "strRuta")]
    pub strruta: Option<String>,
}

// Modelo para la tabla Menu
#[derive(Serialize, Deserialize, FromRow)]
pub struct MenuRow {
    pub id: i32,
    #[serde(rename = "strNombreMenu")]
    pub strnombremenu: String,
}