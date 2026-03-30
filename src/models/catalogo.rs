use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sexo {
    pub id: i32,
    #[serde(rename = "strSexo")] // Para que el JSON se vea igual que en JS/Python
    pub strsexo: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EstadoUsuario {
    pub id: i32,
    #[serde(rename = "strNombreEstado")]
    pub strnombreestado: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Menu {
    pub id: i32,
    #[serde(rename = "strNombreMenu")]
    pub strnombremenu: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PerfilCatalogo {
    pub id: i32,
    #[serde(rename = "strNombrePerfil")]
    pub strnombreperfil: String,
    #[serde(rename = "bitAdministrador")]
    pub bitadministrador: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModuloCatalogo {
    pub id: i32,
    #[serde(rename = "strNombreModulo")]
    pub strnombremodulo: String,
    #[serde(rename = "idMenu")]
    pub idmenu: i32,
    #[serde(rename = "strRuta")]
    pub strruta: Option<String>,
}