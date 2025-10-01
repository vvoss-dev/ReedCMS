use actix_web::{web, HttpRequest, HttpResponse, Result};
use minijinja::{Environment, context};
use std::path::Path;
use chrono::Datelike;
use serde_json;

use super::client::{detect_client_info, parse_screen_info, is_bot_request, generate_screen_detection_html};

/// Creates request info object for templates (needed for language switcher)
fn create_request_info(req: &actix_web::HttpRequest, lang: &str) -> serde_json::Value {
    serde_json::json!({
        "path_info": req.path(),
        "language": lang
    })
}
use super::translations::Translations;
use super::routing::Routing;
use super::components::ComponentRegistry;
use super::pageconf::{PageConfig, make_pageconf_function};
use serde_json::Value;
use crate::CONFIG;
use std::sync::Arc;

/// Render page with language and routing from URL
pub async fn render_with_lang_and_routing(
    req: HttpRequest,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
    template_name: &str,
    current_page: &str,
    lang: &str,
) -> Result<HttpResponse> {
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }
    
    let mut client = detect_client_info(&req);
    
    // Use language from URL
    client.lang = lang.to_string();
    
    // Retrieve latest update date from git log or use current date as fallback
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    
    // Create page info object
    let page_info = serde_json::json!({
        "languages": CONFIG!().get_site_languages(),
        "latest_update": latest_update
    });
    
    // Create MiniJinja environment with filters
    let mut env_clone = env.as_ref().clone();
    env_clone.add_filter("route", super::functions::make_route_filter(routing.as_ref().clone(), lang.to_string()));
    env_clone.add_filter("text", super::functions::make_text_filter(Some(lang.to_string())));

    let template = env_clone.get_template(template_name)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Create configuration object with only needed values (avoid serialising sensitive data)
    let config_for_template = serde_json::json!({
        "dev_mode": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string()) == "DEV"
    });

    let request_info = create_request_info(&req, lang);

    let rendered = template
        .render(context!(
            current_year => chrono::Local::now().year(),
            client => client,
            page => page_info,
            current_page => current_page,
            current_lang => lang,
            config => config_for_template,
            request => request_info
        ))
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Set language cookie
    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("lang", lang.to_string())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        )
        .content_type("text/html")
        .body(rendered))
}

/// Serve static files (CSS, JS, images, fonts)
pub async fn static_files(path: web::Path<String>) -> Result<HttpResponse> {
    // Use configuration-aware public path directly (no page/ prefix needed)
    let public_path = CONFIG!().get_site_public_path();
    let file_path = format!("{}/{}", public_path, path);

    // Debug logging to identify the issue
    log::debug!("Static file request - path: '{}', public_path: '{}', file_path: '{}'", path, public_path, file_path);
    
    // Determine content type based on extension
    let content_type = match Path::new(&path.as_str()).extension().and_then(|s| s.to_str()) {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    };


    // Read and serve the file
    match std::fs::read(&file_path) {
        Ok(contents) => {
            let cache_max_age = CONFIG!().get_site_public_cache_max_age();
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", format!("public, max-age={}", cache_max_age)))
                .body(contents))
        },
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

// Redirect to language-specific URL
pub async fn redirect_to_language(
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Check for language cookie
    let mut lang = "en".to_string(); // Default
    
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookies_str) = cookie_header.to_str() {
            for cookie in cookies_str.split(';') {
                let trimmed = cookie.trim();
                if let Some(value) = trimmed.strip_prefix("lang=") {
                    let cookie_lang = value.to_lowercase();
                    if CONFIG!().get_site_languages().contains(&cookie_lang) {
                        lang = cookie_lang;
                        break;
                    }
                }
            }
        }
    }
    
    // If no cookie, detect from browser
    if lang == "en" {
        if let Some(accept_lang) = req.headers().get("accept-language") {
            if let Ok(lang_str) = accept_lang.to_str() {
                for lang_part in lang_str.split(',') {
                    let detected = lang_part.split(';').next().unwrap_or("").trim();
                    let lang_code = detected[..2.min(detected.len())].to_lowercase();
                    if CONFIG!().get_site_languages().contains(&lang_code) {
                        lang = lang_code;
                        break;
                    }
                }
            }
        }
    }
    
    // Get the current path
    let path = req.path();
    let redirect_url = format!("/{}{}", lang, path);
    
    Ok(HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish())
}

// Route definitions as a macro for DRY
#[macro_export]
macro_rules! page_handler {
    ($name:ident, $template:expr, $page_name:expr) => {
        pub async fn $name(
            req: HttpRequest,
            lang: web::Path<String>,
            env: web::Data<Environment<'static>>,
            translations: web::Data<Translations>,
            routing: web::Data<Routing>,
                ) -> Result<HttpResponse> {
            // Validate language
            let lang_str = lang.into_inner();
            if !CONFIG!().get_site_languages().contains(&lang_str) {
                return Ok(HttpResponse::NotFound().finish());
            }
            
            render_with_lang_and_routing(req, env, translations, routing, $template, $page_name, &lang_str).await
        }
    };
}

// Generate all page handlers
// Using new page templates
// S05-06-Z: Landing page now uses component system
pub async fn index(
    req: HttpRequest,
    path: web::Path<String>,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
    component_registry: web::Data<Arc<ComponentRegistry>>,
) -> Result<HttpResponse> {
    let lang = path.into_inner();

    // Use component system for landing page (empty route)
    if should_use_component_system(&req, "") {
        // S06-03 Phase 3: Use static pageconf scanning for landing page
        if let Some(layout_name) = find_layout_by_route("") {
            // Try to find layout component
            if let Some(component) = component_registry.components.get(&layout_name) {
                log::info!("Rendering landing page via S06-03 static pageconf (layout: {})", layout_name);
                return render_component_page_full(req, component, &lang, "", &component_registry).await;
            } else {
                log::warn!("Landing layout component {} not found in registry", layout_name);
            }
        } else {
            log::warn!("No landing layout found for empty route via S06-03 static pageconf");
        }
    }

    // No fallback! S06-03 only system
    Ok(HttpResponse::NotFound()
        .content_type("text/plain")
        .body("Landing page not found (S06-03 system active)"))
}
// S05-06-Z: These page handlers are replaced by component_aware_page_handler
// page_handler!(portfolio, "content/portfolio.jinja", "portfolio");
// page_handler!(knowledge, "content/knowledge.jinja", "knowledge");

// Generic handler for cluster page subpages using routing system
pub async fn cluster_subpage(
    req: HttpRequest,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
    template_name: &str,
    route_key: &str,
) -> Result<HttpResponse> {
    // Extract language from URL path
    let path = req.path();
    let lang_str = if path.starts_with("/de/") {
        "de"
    } else if path.starts_with("/en/") {
        "en"
    } else {
        "en" // fallback
    };
    
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }
    
    let mut client = detect_client_info(&req);
    
    // Use language from URL
    client.lang = lang_str.to_string();
    
    // Retrieve latest update date from git log or use current date as fallback
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    
    // Get language URLs from routing system
    let lang_urls = routing.get_lang_urls(route_key);
    
    // Create page info object with special URLs for language switching
    let page_info = serde_json::json!({
        "languages": CONFIG!().get_site_languages(),
        "latest_update": latest_update,
        "lang_urls": lang_urls
    });
    
    // Create MiniJinja environment with filters
    let mut env_clone = env.as_ref().clone();
    env_clone.add_filter("route", super::functions::make_route_filter(routing.as_ref().clone(), lang_str.to_string()));
    env_clone.add_filter("text", super::functions::make_text_filter(Some(lang_str.to_string())));

    let template = env_clone.get_template(template_name)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Create configuration object with only needed values (avoid serialising sensitive data)
    let config_for_template = serde_json::json!({
        "dev_mode": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string()) == "DEV"
    });

    // Create request info for templates (needed for language switcher)
    let request_info = serde_json::json!({
        "path_info": req.path(),
        "language": lang_str
    });

    let rendered = template
        .render(context!(
            current_year => chrono::Local::now().year(),
            client => client,
            page => page_info,
            current_page => route_key,
            route_key => route_key,
            current_lang => lang_str,
            config => config_for_template,
            request => request_info
        ))
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Set language cookie
    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("lang", lang_str.to_string())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        )
        .content_type("text/html")
        .body(rendered))
}

// Generic handler for any cluster subpage - determines route_key from URL
pub async fn generic_page_handler(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
) -> Result<HttpResponse> {
    let (lang, page) = path.into_inner();
    
    // Find the route key from the path
    if let Some(key) = routing.get_key_from_path_and_lang(&page, &lang) {
        // Check if this is a single page (not a cluster subpage)
        if !key.contains('.') {
            // Determine template path
            let template = format!("content/{}.jinja", key);
            
            // Use render_with_lang_and_routing for consistency
            render_with_lang_and_routing(
                req,
                env,
                translations,
                routing,
                &template,
                &key,
                &lang,
            ).await
        } else {
            // This is a cluster subpage, should not be handled here
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

pub async fn generic_cluster_handler(
    req: HttpRequest,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
) -> Result<HttpResponse> {
    // Get route_key from URL path
    let path = req.path();
    let route_key = routing.get_key_from_path(path)
        .ok_or_else(|| actix_web::error::ErrorNotFound("Route not found"))?;
    
    // Determine template name from route_key
    // knowledge.bastille -> content/knowledge-bastille.jinja
    let template_name = format!("content/{}.jinja", route_key.replace('.', "-"));
    
    cluster_subpage(
        req,
        env,
        translations,
        routing,
        &template_name,
        &route_key
    ).await
}
page_handler!(impressum, "content/impressum.jinja", "impressum");
page_handler!(blog, "content/blog.jinja", "blog");
page_handler!(contact, "content/contact.jinja", "contact");

/// Serve CSS/JS bundle files from public/bundles directory
pub async fn serve_bundle_assets(path: web::Path<String>) -> Result<HttpResponse> {
    let bundle_path = path.into_inner();
    let file_path = format!("public/session/{}", bundle_path);

    // Security: Only serve CSS and JS files from bundles directory
    if !bundle_path.ends_with(".css") && !bundle_path.ends_with(".js") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Security: Prevent directory traversal
    if bundle_path.contains("..") || bundle_path.contains("/") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Determine content type
    let content_type = if bundle_path.ends_with(".css") {
        "text/css"
    } else if bundle_path.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    };

    // Try to read and serve the file
    match std::fs::read_to_string(&file_path) {
        Ok(contents) => {
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", "public, max-age=31536000")) // 1 year
                .insert_header(("ETag", format!("\"{}\"", generate_etag(&contents))))
                .body(contents))
        }
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

/// Serve component assets from templates directory (development mode)
pub async fn serve_component_assets(path: web::Path<String>) -> Result<HttpResponse> {
    let asset_path = path.into_inner();
    let file_path = format!("templates/{}", asset_path);

    // Security: Prevent directory traversal
    if asset_path.contains("..") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Security: Only serve CSS and JS files from allowed component directories
    if !asset_path.starts_with("layouts/")
        && !asset_path.starts_with("components/atoms/")
        && !asset_path.starts_with("components/molecules/")
        && !asset_path.starts_with("components/organisms/") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Only serve CSS and JS files
    if !asset_path.ends_with(".css") && !asset_path.ends_with(".js") {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // Determine content type
    let content_type = if asset_path.ends_with(".css") {
        "text/css"
    } else if asset_path.ends_with(".js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    };

    // Try to read and serve the file
    match std::fs::read_to_string(&file_path) {
        Ok(contents) => {
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", "no-cache, no-store, must-revalidate")) // Development mode - no cache
                .body(contents))
        }
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

/// Generate ETag for content (simple hash)
fn generate_etag(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// ============================================================================
// FEATURE FLAG SYSTEM - S05-06-D
// ============================================================================

/// Check if component test mode should be enabled
fn should_use_component_system(req: &HttpRequest, page_route: &str) -> bool {
    // Check for explicit test flag in query string
    if req.query_string().contains("test=1") {
        return true;
    }

    // Check environment variable for global component mode
    if std::env::var("COMPONENT_SYSTEM_ACTIVE").is_ok() {
        return true;
    }

    // Check for specific route migration completion
    if let Ok(migrated_routes) = std::env::var("MIGRATED_ROUTES") {
        if migrated_routes.split(',')
            .any(|route| route.trim() == page_route) {
            return true;
        }
    }

    // S05-06-Z: Component system is now production default
    true
}

/// Determine component template path from page route
fn get_component_template_path(page_route: &str) -> Option<String> {
    // Simple mapping for S05-06-D implementation
    match page_route {
        "impressum" => Some("layouts/impressum/impressum.jinja".to_string()),
        "portfolio" => Some("layouts/portfolio/portfolio.jinja".to_string()),
        _ => None,
    }
}

/// Render component-based page
async fn render_component_page(
    req: HttpRequest,
    component_template: &str,
    page_route: &str,
    lang: &str,
) -> Result<HttpResponse> {
    // For S05-06-D, we create a minimal component-aware environment
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }

    let mut client = detect_client_info(&req);
    client.lang = lang.to_string();

    // Retrieve latest update date
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    // Create page info object
    let page_info = serde_json::json!({
        "languages": CONFIG!().get_site_languages(),
        "latest_update": latest_update
    });

    // Create minimal MiniJinja environment for component templates
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));

    // Add basic filters with language support
    env.add_filter("text", super::functions::make_text_filter(Some(lang.to_string())));

    let template = env.get_template(component_template)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Component template not found: {} - {}", component_template, e)))?;

    // Create configuration object
    let config_for_template = serde_json::json!({
        "dev_mode": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string()) == "DEV",
        "component_system": true,
        "session_hash": CONFIG!().get_session_hash()
    });

    // Extract layout name from component template path (e.g., "layouts/impressum/impressum.jinja" -> "impressum")
    let layout_name = component_template
        .split('/')
        .nth(1)
        .unwrap_or("page");

    let request_info = create_request_info(&req, lang);

    let rendered = template
        .render(context!(
            current_year => chrono::Local::now().year(),
            client => client,
            page => page_info,
            current_page => page_route,
            current_lang => lang,
            layout_name => layout_name,
            config => config_for_template,
            request => request_info
        ))
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Component rendering failed: {}", e)))?;

    // Set language cookie
    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("lang", lang.to_string())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        )
        .content_type("text/html")
        .body(rendered))
}

/// Component-aware generic page handler with S06-03 pageconf integration
pub async fn component_aware_page_handler(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    _legacy_env: web::Data<Environment<'static>>,
    component_registry: web::Data<Arc<ComponentRegistry>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
) -> Result<HttpResponse> {
    let (lang, page_route) = path.into_inner();

    // Validate language
    if !CONFIG!().get_site_languages().contains(&lang) {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Check if component system should be used
    if should_use_component_system(&req, &page_route) {
        // S06-03 Phase 3: Use static pageconf scanning (replaces routing.csv)
        if let Some(layout_name) = find_layout_by_route(&page_route) {
            // Try to find layout component
            if let Some(component) = component_registry.components.get(&layout_name) {
                log::info!("Rendering {} via S06-03 static pageconf (layout: {})", page_route, layout_name);
                return render_component_page_full(req, component, &lang, &page_route, &component_registry).await;
            } else {
                log::warn!("Layout component {} not found in registry for route {}", layout_name, page_route);
            }
        } else {
            log::warn!("No layout found for route {} via S06-03 static pageconf", page_route);
        }
    }

    // No fallback! S06-03 only system
    Ok(HttpResponse::NotFound()
        .content_type("text/plain")
        .body(format!("Page not found: {} (S06-03 system active)", page_route)))
}

/// Feature flag-aware generic page handler (backward compatibility)
pub async fn generic_page_handler_with_feature_flag(
    req: HttpRequest,
    path: web::Path<(String, String)>,
    env: web::Data<Environment<'static>>,
    translations: web::Data<Translations>,
    routing: web::Data<Routing>,
) -> Result<HttpResponse> {
    let (lang, page) = path.into_inner();

    // Validate language
    if !CONFIG!().get_site_languages().contains(&lang) {
        return Ok(HttpResponse::NotFound().finish());
    }

    // Check if component system should be used
    if should_use_component_system(&req, &page) {
        if let Some(component_template) = get_component_template_path(&page) {
            log::info!("Rendering {} via component system (template: {})", page, component_template);
            return render_component_page(req, &component_template, &page, &lang).await;
        } else {
            log::warn!("Component template not found for {}, falling back to legacy", page);
        }
    }

    // Fallback to legacy system
    log::info!("Rendering {} via legacy system", page);

    // Recreate path for legacy handler
    let legacy_path = web::Path::from((lang, page));
    generic_page_handler(req, legacy_path, env, translations, routing).await
}

// ============================================================================
// S05-06-E: COMPONENT REGISTRY INTEGRATION
// ============================================================================

/// Find component by route using ComponentRegistry and global routing
fn find_component_by_route<'a>(
    registry: &'a ComponentRegistry,
    routing: &super::routing::Routing,
    page_route: &str,
    lang: &str,
) -> Option<&'a super::components::Component> {
    // Use routing registry to find route -> pagekey mapping
    let _route_key_format = format!("ROUTE_{}", lang.to_uppercase());

    // Search through routing registry (environment-aware format)
    if let Some(lang_map) = routing.routes.get(&lang.to_uppercase()) {
        for (key, value) in lang_map {
            if key.starts_with("ROUTE_") && value == page_route {
                // Extract pagekey from ROUTE_KNOWLEDGE -> knowledge
                if let Some(pagekey) = key.strip_prefix("ROUTE_") {
                    let layout_name = pagekey.to_lowercase();
                    // Look for layout component
                    if let Some(component) = registry.components.get(&layout_name) {
                        return Some(component);
                    }
                }
            }
        }
    }
    None
}

/// Build reed context with auto-resolution for template access
///
/// Implements S06-03 Phase 2: Template Context Auto-Resolution
/// Provides both local access (reed.pagekey) and scoped access (reed.pagekey@landing)
/// following the @DEV/@PROD/@DEFAULT keybase pattern
fn build_reed_context_for_layout(layout_name: &str) -> Value {
    // Use layout name as pagekey for auto-resolution
    crate::libs::pageconf::build_reed_context_for_pagekey(layout_name)
}

/// Find layout by route using pageroute@pagekey scanning
///
/// Implements S06-03 Phase 3: Route Resolution Logic
/// Replaces find_component_by_route() with pageconf() scoped access
/// Scans all pagekeys for matching routes: pageroute@pagekey == page_route

/// Find layout by route using static pageconf scanning (S06-03 replacement for routing.csv)
///
/// Uses startup-time scanned pageconf() calls instead of runtime PageConfig
/// This avoids circular dependency: route → layout → pageconf() → route
fn find_layout_by_route(page_route: &str) -> Option<String> {
    // Use static scanned mapping from startup
    let route_mappings = super::pageconf::get_global_route_mappings();

    // Handle ROOT route mapping: empty route "" maps to "ROOT" key
    let lookup_key = if page_route.is_empty() { "ROOT" } else { page_route };
    log::debug!("S06-03: Looking up route '{}' with key '{}' in route mappings", page_route, lookup_key);
    log::debug!("S06-03: Route mappings size: {}", route_mappings.len());
    log::debug!("S06-03: Route mappings keys: {:?}", route_mappings.keys().collect::<Vec<_>>());

    let result = route_mappings.get(lookup_key).cloned();
    if let Some(ref layout) = result {
        log::debug!("S06-03: Found layout '{}' for route '{}'", layout, page_route);
    } else {
        log::debug!("S06-03: No layout found for route '{}' (key: '{}')", page_route, lookup_key);
    }
    result
}

/// Enhanced component page rendering with ComponentRegistry
async fn render_component_page_full(
    req: HttpRequest,
    component: &super::components::Component,
    lang: &str,
    page_route: &str,
    _registry: &Arc<ComponentRegistry>,
) -> Result<HttpResponse> {
    // Skip detection for bots
    if !is_bot_request(&req) && parse_screen_info(&req).is_none() {
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(generate_screen_detection_html()));
    }

    let mut client = detect_client_info(&req);
    client.lang = lang.to_string();

    // Retrieve latest update date
    let latest_update = std::process::Command::new("git")
        .args(&["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    // Create page info object
    let page_info = serde_json::json!({
        "languages": CONFIG!().get_site_languages(),
        "latest_update": latest_update
    });

    // Create MiniJinja environment with multi-path component template loader
    let mut env = Environment::new();

    // S05-06-Z: Global project-root template loader
    env.set_loader(move |name| {
        // Always search from project root - no more relative path issues
        if let Ok(content) = std::fs::read_to_string(name) {
            return Ok(Some(content));
        }
        Ok(None)
    });

    // Add robust device-aware component template functions with fallback
    let client_interaction_mode = client.interaction_mode.clone();

    env.add_function("organism", move |name: String| -> String {
        let base_path = CONFIG!().get_site_organisms_path();
        let device = &client_interaction_mode;

        // 1. Try device-specific template
        let device_template = format!("{}/{}/{}.{}.jinja", base_path, name, name, device);
        if std::path::Path::new(&device_template).exists() {
            return device_template;
        }

        // 2. Touch fallback (mobile-first - should always work)
        let touch_fallback = format!("{}/{}/{}.touch.jinja", base_path, name, name);
        if std::path::Path::new(&touch_fallback).exists() {
            return touch_fallback;
        }

        // 3. Legacy fallback (migration support)
        format!("{}/{}/{}.jinja", base_path, name, name)
    });

    let client_interaction_mode_molecule = client.interaction_mode.clone();
    env.add_function("molecule", move |name: String| -> String {
        let base_path = CONFIG!().get_site_molecules_path();
        let device = &client_interaction_mode_molecule;

        // 1. Try device-specific template
        let device_template = format!("{}/{}/{}.{}.jinja", base_path, name, name, device);
        if std::path::Path::new(&device_template).exists() {
            return device_template;
        }

        // 2. Touch fallback (mobile-first - should always work)
        let touch_fallback = format!("{}/{}/{}.touch.jinja", base_path, name, name);
        if std::path::Path::new(&touch_fallback).exists() {
            return touch_fallback;
        }

        // 3. Legacy fallback (migration support)
        format!("{}/{}/{}.jinja", base_path, name, name)
    });

    let client_interaction_mode_atom = client.interaction_mode.clone();
    env.add_function("atom", move |name: String| -> String {
        let base_path = CONFIG!().get_site_atoms_path();
        let device = &client_interaction_mode_atom;

        // 1. Try device-specific template
        let device_template = format!("{}/{}/{}.{}.jinja", base_path, name, name, device);
        if std::path::Path::new(&device_template).exists() {
            return device_template;
        }

        // 2. Touch fallback (mobile-first - should always work)
        let touch_fallback = format!("{}/{}/{}.touch.jinja", base_path, name, name);
        if std::path::Path::new(&touch_fallback).exists() {
            return touch_fallback;
        }

        // 3. Legacy fallback (migration support)
        format!("{}/{}/{}.jinja", base_path, name, name)
    });

    // Layouts also get fallback support now
    let client_interaction_mode_layout = client.interaction_mode.clone();
    env.add_function("layout", move |name: String| -> String {
        let base_path = CONFIG!().get_site_layout_path();
        let device = &client_interaction_mode_layout;

        // 1. Try device-specific layout template
        let device_template = format!("{}/{}/{}.{}.jinja", base_path, name, name, device);
        if std::path::Path::new(&device_template).exists() {
            return device_template;
        }

        // 2. Touch fallback for layouts
        let touch_fallback = format!("{}/{}/{}.touch.jinja", base_path, name, name);
        if std::path::Path::new(&touch_fallback).exists() {
            return touch_fallback;
        }

        // 3. Standard layout (current pattern)
        format!("{}/{}/{}.jinja", base_path, name, name)
    });

    // Create page configuration storage for reed.* access
    let page_config = PageConfig::new();

    // Add pageconf() function for layout templates
    env.add_function("pageconf", make_pageconf_function(page_config.clone()));

    // Add icon function
    env.add_function("icon", crate::libs::icons::load_icon);

    // Add text filter with language support
    env.add_filter("text", super::functions::make_text_filter(Some(lang.to_string())));

    // Determine template path
    let template_path = format!("{}/{}.jinja", component.info.path, component.data.name);

    let template = env.get_template(&template_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(
            format!("Component template not found: {} - {}", template_path, e)
        ))?;

    // Create enhanced configuration object
    let config_for_template = serde_json::json!({
        "dev_mode": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "PROD".to_string()) == "DEV",
        "component_system": true,
        "component_name": component.data.name,
        "session_hash": CONFIG!().get_session_hash()
    });

    let request_info = create_request_info(&req, lang);

    let rendered = template
        .render(context!(
            current_year => chrono::Local::now().year(),
            client => client,
            page => page_info,
            current_page => component.data.name,
            current_lang => lang,
            layout_name => component.data.name,
            config => config_for_template,
            request => request_info,
            reed => {
                // Get pagekey from route for proper auto-resolution
                if let Some(pagekey) = crate::libs::pageconf::get_pagekey_from_route(page_route) {
                    crate::libs::pageconf::build_reed_context_for_pagekey(&pagekey)
                } else {
                    // Fallback to layout name as pagekey
                    crate::libs::pageconf::build_reed_context_for_pagekey(&component.data.name)
                }
            }
        ))
        .map_err(|e| actix_web::error::ErrorInternalServerError(
            format!("Component rendering failed: {}", e)
        ))?;

    // Set language cookie
    Ok(HttpResponse::Ok()
        .cookie(
            actix_web::cookie::Cookie::build("lang", lang.to_string())
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(365))
                .same_site(actix_web::cookie::SameSite::Lax)
                .finish()
        )
        .content_type("text/html")
        .body(rendered))
}