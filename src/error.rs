use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
	#[error("Configuration error")]
	Config(#[from] confy::ConfyError),
	#[error("IO error")]
	Io(#[from] std::io::Error),
	#[error("Blocking error")]
	Blocking(#[from] actix_web::error::BlockingError),
	#[error("Multipart error")]
	Multipart(#[from] actix_multipart::MultipartError),
	#[error("unknown error")]
	Unknown,
}
