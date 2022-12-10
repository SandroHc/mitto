use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("configuration error: {0}")]
	Config(#[from] confy::ConfyError),
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),
	#[error("blocking error: {0}")]
	Blocking(#[from] actix_web::error::BlockingError),
	#[error("multipart error: {0}")]
	Multipart(#[from] actix_multipart::MultipartError),
	#[error(transparent)]
	Unknown(#[from] Box<dyn std::error::Error + Send>),
}
