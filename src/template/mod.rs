use std::collections::HashMap;
use std::sync::OnceLock;
use tera::{Tera, Value};
use thiserror::Error;

use crate::schema::CvProfile;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Tera rendering error: {0}")]
    Tera(#[from] tera::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Embedded default Star Rover Tera template
pub const STAR_ROVER_TEMPLATE: &str = include_str!("../../templates/star_rover.tex.tera");

static TERA_INSTANCE: OnceLock<Tera> = OnceLock::new();

/// Escape standard LaTeX special characters so user content never breaks the TeX parser.
pub fn escape_latex_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '$' => out.push_str("\\$"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '\\' => out.push_str("\\textbackslash{}"),
            _ => out.push(c),
        }
    }
    out
}

/// Custom Tera filter to safely escape LaTeX strings.
fn latex_escape_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(escape_latex_str(s))),
        Value::Array(arr) => {
            let escaped_arr: Vec<Value> = arr
                .iter()
                .map(|item| {
                    if let Value::String(s) = item {
                        Value::String(escape_latex_str(s))
                    } else {
                        item.clone()
                    }
                })
                .collect();
            Ok(Value::Array(escaped_arr))
        }
        _ => Ok(value.clone()),
    }
}

/// Initialize Tera template engine with custom filters.
pub fn get_tera() -> &'static Tera {
    TERA_INSTANCE.get_or_init(|| {
        let mut tera = Tera::default();
        tera.register_filter("latex_escape", latex_escape_filter);
        tera.add_raw_template("star_rover.tex", STAR_ROVER_TEMPLATE)
            .expect("Failed to parse embedded star_rover.tex.tera template");
        tera
    })
}

/// Render a canonical CvProfile into a fully valid LaTeX source string.
pub fn render_latex(profile: &CvProfile) -> Result<String, TemplateError> {
    let tera = get_tera();
    let context = tera::Context::from_serialize(profile)?;
    let rendered = tera.render("star_rover.tex", &context)?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::sample_cv_profile;

    #[test]
    fn test_escape_latex() {
        assert_eq!(
            escape_latex_str("C++ & C# 100% $500_000 {cool} ^ ~ \\"),
            "C++ \\& C\\# 100\\% \\$500\\_000 \\{cool\\} \\textasciicircum{} \\textasciitilde{} \\textbackslash{}"
        );
    }

    #[test]
    fn test_render_sample_profile() {
        let profile = sample_cv_profile();
        let rendered = render_latex(&profile).expect("Rendering failed");
        assert!(rendered.contains("STAR ROVER"));
        assert!(rendered.contains("Amazon"));
        assert!(rendered.contains("\\section{Experience}"));
    }
}
