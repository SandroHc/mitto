use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web_httpauth::extractors::{basic, AuthenticationError};

use crate::AppConfig;

/// Middleware that checks that each request has an "Authorization" header and that it matches the auth token.
pub async fn validator(
    req: ServiceRequest,
    credentials: basic::BasicAuth,
) -> Result<ServiceRequest, (actix_web::error::Error, ServiceRequest)> {
    if let Some(password) = credentials.password() {
        if let Some(app_config) = req.app_data::<web::Data<AppConfig>>() {
            if password == app_config.auth_token {
                return Ok(req);
            }
        }
    }

    Err((
        AuthenticationError::from(basic::Config::default()).into(),
        req,
    ))
}
