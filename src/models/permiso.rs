use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PermisosPerfil {
    pub id: Option<i32>,
    
    #[serde(rename = "idModulo")]
    pub idmodulo: Option<i32>,
    
    #[serde(rename = "idPerfil")]
    pub idperfil: Option<i32>,
    
    // 👇 NUEVO: Campo para el nombre del módulo
    #[sqlx(default)]
    #[serde(rename = "strNombreModulo", skip_serializing_if = "Option::is_none")]
    pub strnombremodulo: Option<String>,
    
    #[serde(rename = "bitAgregar")]
    pub bitagregar: bool,
    
    #[serde(rename = "bitEditar")]
    pub biteditar: bool,
    
    #[serde(rename = "bitEliminar")]
    pub biteliminar: bool,
    
    #[serde(rename = "bitConsulta")]
    pub bitconsulta: bool,
    
    #[serde(rename = "bitDetalle")]
    pub bitdetalle: bool,
}

// Estructura opcional para replicar exactamente el comportamiento de to_dict()
// que anida los booleanos dentro de un objeto "permisos"
#[derive(Serialize)]
pub struct PermisosPerfilOutput {
    pub id: Option<i32>,
    #[serde(rename = "idModulo")]
    pub idmodulo: Option<i32>,
    #[serde(rename = "idPerfil")]
    pub idperfil: Option<i32>,
    pub permisos: DetallePermisos,
}

#[derive(Serialize)]
pub struct DetallePermisos {
    pub agregar: bool,
    pub editar: bool,
    pub eliminar: bool,
    pub consulta: bool,
    pub detalle: bool,
}

impl PermisosPerfil {
    /// Replica la lógica de to_dict() de Python transformando la estructura plana
    /// en una estructura con permisos anidados.
    pub fn into_output(self) -> PermisosPerfilOutput {
        PermisosPerfilOutput {
            id: self.id,
            idmodulo: self.idmodulo,
            idperfil: self.idperfil,
            permisos: DetallePermisos {
                agregar: self.bitagregar,
                editar: self.biteditar,
                eliminar: self.biteliminar,
                consulta: self.bitconsulta,
                detalle: self.bitdetalle,
            },
        }
    }
}