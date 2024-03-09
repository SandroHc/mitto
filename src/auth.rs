use actix_web::dev::ServiceRequest;
use actix_web::web::Data;
use actix_web_httpauth::extractors::{basic, AuthenticationError};
use tracing::trace;

use crate::AppConfig;

/// Middleware that checks that each request has an "Authorization" header and that it matches the auth token.
pub async fn validator(
    req: ServiceRequest,
    credentials: basic::BasicAuth,
) -> Result<ServiceRequest, (actix_web::error::Error, ServiceRequest)> {
    let auth_token = req
        .app_data::<Data<AppConfig>>()
        .map(|app_config| app_config.auth_token.as_str())
        .filter(|auth_token| !auth_token.is_empty());

    match auth_token {
        None => {
            trace!("Bypassing auth; no auth token configured");
            Ok(req)
        }
        Some(auth_token) => {
            if let Some(password) = credentials.password() {
                if password == auth_token {
                    trace!("Request passed authentication");
                    Ok(req)
                } else {
                    trace!("Request failed authentication; invalid authorization token");
                    Err((
                        AuthenticationError::from(basic::Config::default()).into(),
                        req,
                    ))
                }
            } else {
                trace!("Request failed authentication; authorization header not present");
                Err((
                    AuthenticationError::from(basic::Config::default()).into(),
                    req,
                ))
            }
        }
    }
}
