use minijinja::{Error as MiniJinjaError, Value as MiniJinjaValue};
use crate::libs::routing::Routing;

/// Creates a route filter for getting URLs in different languages using modern routing system
pub fn make_route_filter(
    routing: Routing,
    current_lang: String,
) -> impl Fn(&MiniJinjaValue, Option<&MiniJinjaValue>) -> Result<MiniJinjaValue, MiniJinjaError> + Send + Sync + 'static {
    move |value: &MiniJinjaValue, lang_param: Option<&MiniJinjaValue>| -> Result<MiniJinjaValue, MiniJinjaError> {
        // Get the page key from the value (e.g., "knowledge")
        let page_key = value.as_str()
            .ok_or_else(|| MiniJinjaError::new(minijinja::ErrorKind::InvalidOperation, "route filter requires a page key"))?;

        // Get the target language parameter or use current language as default
        let target_lang = if let Some(lang_val) = lang_param {
            let lang_str = lang_val.as_str()
                .ok_or_else(|| MiniJinjaError::new(minijinja::ErrorKind::InvalidOperation, "route filter lang parameter must be a string"))?;
            if lang_str == "auto" {
                &current_lang
            } else {
                lang_str
            }
        } else {
            &current_lang
        };

        // Convert page_key to ROUTE_* format and look up route
        let route_key = format!("ROUTE_{}", page_key.to_uppercase());
        if let Some(path) = routing.get_path(&route_key, target_lang) {
            return Ok(MiniJinjaValue::from(path));
        }

        // Handle special cases for homepage
        if page_key == "landing" || page_key == "index" || page_key.is_empty() {
            if let Some(path) = routing.get_path("ROUTE_INDEX", target_lang) {
                return Ok(MiniJinjaValue::from(path));
            }
        }

        // Fallback to constructing basic URL if route not found
        let fallback_url = if page_key.is_empty() {
            format!("/{}/", target_lang)
        } else {
            format!("/{}/{}", target_lang, page_key)
        };
        Ok(MiniJinjaValue::from(fallback_url))
    }
}

/// Creates a unified text filter for getting translated strings using TEXT!() registry
/// Supports explicit language (de, en) and auto-detection (auto/AUTO)
/// Usage: {{ 'KEY' | text('de') }} or {{ 'KEY' | text('auto') }}
pub fn make_text_filter(
    detected_lang: Option<String>,
) -> impl Fn(&str, Option<&str>) -> String + Send + Sync + 'static {
    let current_lang = detected_lang.unwrap_or_else(|| "de".to_string());

    move |key: &str, lang_param: Option<&str>| -> String {
        let language = match lang_param {
            Some("auto") | Some("AUTO") => &current_lang, // Use detected language from URL/cookie
            Some(explicit_lang) => explicit_lang, // Use explicit language (de, en, etc.)
            None => &current_lang, // Default to detected language if no param
        };

        // Use TEXT!() registry with @DE/@EN/@AUTO pattern and automatic fallback
        crate::TEXT!().get_with_language_or(key, language, key)
    }
}