use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;

use crate::CONFIG;

// Simple validator using ultimate flexible configuration system
pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Check if auth is enabled
    if !CONFIG!().get_server_auth_enabled() {
        return Ok(req);
    }

    // Check credentials against configuration
    match (credentials.user_id(), credentials.password()) {
        (user, Some(pass)) if user == CONFIG!().get_server_auth_username() && pass == CONFIG!().get_server_auth_password() => {
            Ok(req)
        }
        _ => {
            let challenge = Basic::default();
            Err((AuthenticationError::new(challenge).into(), req))
        }
    }
}