// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;

use super::{TransformOptions, TransformResult};

/// Transform CSS to JavaScript injection code
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_css(source: &str, _options: &TransformOptions) -> Result<TransformResult> {
    tracing::debug!("Transforming CSS to JS injection code");

    let escaped_css = source
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let injection_code = format!(
        r#"// CSS Module Injection
(function() {{
  if (typeof document !== 'undefined') {{
    var style = document.createElement('style');
    style.textContent = `{}`;
    document.head.appendChild(style);
  }}
}})();
"#,
        escaped_css
    );

    Ok(TransformResult {
        code: injection_code,
        source_map: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_injection_code() {
        let source = ".test { color: red; }";
        let options = TransformOptions::default();
        let result = transform_css(source, &options).unwrap();

        assert!(result.code.contains("createElement('style')"));
        assert!(result.code.contains("appendChild"));
        assert!(result.code.contains(".test { color: red; }"));
    }

    #[test]
    fn test_css_escaping() {
        let source = r#".test { content: "hello `world` ${foo}"; }"#;
        let options = TransformOptions::default();
        let result = transform_css(source, &options).unwrap();

        assert!(result.code.contains("\\`"));
        assert!(result.code.contains("\\${"));
    }
}
// CODEGEN-END
