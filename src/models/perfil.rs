use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Perfil {
    pub id: i32,
    #[serde(rename = "strNombrePerfil")]
    pub strnombreperfil: String,
     #[serde(rename = "bitAdministrador")]
    pub bitadministrador: bool,
}