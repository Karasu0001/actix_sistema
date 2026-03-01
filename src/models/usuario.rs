use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Usuario {
    // Añadimos esta directiva para manejar strings vacíos desde el JS
    #[serde(default, deserialize_with = "deserialize_optional_i32")]
    pub id: Option<i32>,
    pub usuario: String,
    pub email: String,
    pub password: Option<String>,
    #[serde(skip_deserializing)] 
    pub created_at: Option<String>,
}

// Función auxiliar para convertir "" en None
fn deserialize_optional_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match s {
        serde_json::Value::Number(n) => Ok(n.as_i64().map(|v| v as i32)),
        serde_json::Value::String(ref s) if s.is_empty() => Ok(None),
        serde_json::Value::Null => Ok(None),
        _ => Ok(None), // Cualquier otro caso (como strings con texto) lo ponemos como None
    }
}