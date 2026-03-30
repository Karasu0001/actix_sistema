// src/models/auth.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // El ID del usuario o username
    pub exp: usize,  // Expiración
}

#[derive(Serialize)]
pub struct Breadcrumb {
    pub name: String,
    pub url: String,
}

// Función auxiliar para generar breadcrumbs basada en la ruta
pub fn generate_breadcrumbs(path: &str) -> Vec<Breadcrumb> {
    let mut breadcrumbs = vec![Breadcrumb {
        name: "My Project".to_string(),
        url: "/".to_string(),
    }];
    
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current_url = String::new();
    
    for part in parts {
        current_url.push_str("/");
        current_url.push_str(part);
        
        let name = part.replace('_', " ").replace('-', " ");
        // Capitalizar la primera letra de cada palabra (implementación simple)
        let capitalized = name.split_whitespace()
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        breadcrumbs.push(Breadcrumb {
            name: capitalized,
            url: current_url.clone(),
        });
    }
    breadcrumbs
}