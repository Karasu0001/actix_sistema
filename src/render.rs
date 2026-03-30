use tera::{Tera, Context};

pub fn init_templates() -> Tera {
    match Tera::new("src/templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error cargando templates: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn render_view(tera: &Tera, template: &str, context: Context) -> String {
    match tera.render(template, &context) {
        Ok(s) => s,
        Err(e) => {
            // Esto mandará el detalle exacto a tu consola (línea que falla, variable faltante, etc.)
            println!("🔥 ERROR DETALLADO DE TERA EN '{}':\n{:#?}", template, e);
            format!("Error renderizando {}: {:#?}", template, e)
        }
    }
}