use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web_httpauth::extractors::{AuthenticationError, basic};
use actix_web_httpauth::extractors::basic::{BasicAuth};
use crate::AppConfig;

pub async fn validator(
	req: ServiceRequest,
	credentials: BasicAuth,
) -> Result<ServiceRequest, (actix_web::error::Error, ServiceRequest)> {
	if let Some(password) = credentials.password() {
		if let Some(app_config) = req.app_data::<web::Data<AppConfig>>() {
			if password == app_config.auth_token {
				return Ok(req);
			}
		}
	}

	Err((AuthenticationError::from(basic::Config::default()).into(), req))
}