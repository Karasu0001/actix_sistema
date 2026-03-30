use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{NaiveDate, NaiveDateTime};

#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct Usuario {
    pub id: i32,
    pub nombre: String,

    #[sqlx(rename = "apellido_p")]
    #[serde(rename = "apellidoP")]
    pub apellidop: Option<String>,

    #[sqlx(rename = "apellido_m")]
    #[serde(rename = "apellidoM")]
    pub apellidom: Option<String>,

    #[sqlx(rename = "id_perfil")]
    #[serde(rename = "idPerfil")]
    pub idperfil: Option<i32>,

    #[sqlx(rename = "str_pwd")]
    #[serde(rename = "strPwd")]
    pub strpwd: Option<String>,

    #[sqlx(rename = "fecha_nacimiento")]
    #[serde(rename = "fechaNacimiento")]
    pub fechanacimiento: Option<NaiveDate>,

    #[sqlx(rename = "id_estado_usuario")]
    #[serde(rename = "idEstadoUsuario")]
    pub idestadousuario: Option<i32>,

    #[sqlx(rename = "id_sexo")]
    #[serde(rename = "idSexo")]
    pub idsexo: Option<i32>,

    #[sqlx(rename = "str_correo")]
    #[serde(rename = "strCorreo")]
    pub strcorreo: Option<String>,

    #[sqlx(rename = "str_numero_celular")]
    #[serde(rename = "strNumeroCelular")]
    pub strnumerocelular: Option<String>,

    #[sqlx(rename = "str_imagen_path")]
    #[serde(rename = "strImagenPath")]
    pub strimagenpath: Option<String>,

    #[sqlx(rename = "fecha_registro")]
    #[serde(rename = "fechaRegistro")]
    pub fecharegistro: Option<NaiveDateTime>,
}

// 🎯 Payload del JWT (lo que guardamos en el token)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPayload {
    pub id: i32,
    pub email: String,
    pub nombre_completo: String,
    pub id_perfil: i32,
    pub iniciales: String,
    pub imagen_path: Option<String>, // 👈 NUEVO: Para el avatar
    pub exp: i64, // timestamp de expiración
}

impl Usuario {
    /// Genera las iniciales del usuario
    pub fn get_iniciales(&self) -> String {
        let primera_nombre = self.nombre.chars().next().unwrap_or('U');
        let primera_apellido = self.apellidop
            .as_ref()
            .and_then(|ap| ap.chars().next())
            .unwrap_or('U');
        
        format!("{}{}", primera_nombre, primera_apellido).to_uppercase()
    }

    /// Genera el nombre completo
    pub fn get_nombre_completo(&self) -> String {
        let mut partes = vec![self.nombre.clone()];
        
        if let Some(ap) = &self.apellidop {
            if !ap.is_empty() {
                partes.push(ap.clone());
            }
        }
        
        if let Some(am) = &self.apellidom {
            if !am.is_empty() {
                partes.push(am.clone());
            }
        }
        
        partes.join(" ")
    }

    /// Convierte el Usuario a UserPayload para JWT
    pub fn to_payload(&self, exp: i64) -> UserPayload {
        UserPayload {
            id: self.id,
            email: self.strcorreo.clone().unwrap_or_default(),
            nombre_completo: self.get_nombre_completo(),
            id_perfil: self.idperfil.unwrap_or(1),
            iniciales: self.get_iniciales(),
            imagen_path: self.strimagenpath.clone(),
            exp,
        }
    }
}