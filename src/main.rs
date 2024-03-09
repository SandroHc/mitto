use std::io::Write;
use std::path::{Path, PathBuf};

use actix_multipart::{Field, Multipart};
use actix_web::{
    get, http::header, http::header::ContentType, post, web, web::Data, App, HttpRequest,
    HttpResponse, HttpServer, Responder,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, level_filters::LevelFilter, trace};
use tracing_subscriber::EnvFilter;

use crate::error::AppError;

mod auth;
mod error;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppConfig {
    /// The address where Mitto will listen on. Usually "127.0.0.1" for listening on the host only
    /// or "0.0.0.0" to listen on every interface.
    listen_address: String,
    /// The port where Mitto will listen on.
    listen_port: u16,
    /// The URL where the service will be available.
    public_url: String,
    /// The directory to store the uploaded files.
    upload_dir: PathBuf,
    /// The authentication token, or password.
    auth_token: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            listen_address: "127.0.0.1".to_string(),
            listen_port: 8080,
            public_url: "http://localhost:8080".to_string(),
            upload_dir: PathBuf::from("./media"),
            auth_token: "".to_string(),
        }
    }
}

#[derive(Serialize)]
struct UploadedFile {
    name: String,
    url: String,
    delete_url: String,
}

#[post("/upload")]
async fn upload(
    req: HttpRequest,
    mut multipart: Multipart,
    state: Data<AppConfig>,
) -> impl Responder {
    debug!("Processing upload request");
    if let Some(Ok(field)) = multipart.next().await {
        let res = save_file(field, &state.upload_dir).await;
        match res {
            Ok(filename) => create_response(&req, state, filename),
            Err(error) => HttpResponse::InternalServerError()
                .content_type(ContentType::plaintext())
                .body(format!("{error}")),
        }
    } else {
        trace!("Upload request didn't have file");
        HttpResponse::BadRequest().finish()
    }
}

fn create_response(req: &HttpRequest, state: Data<AppConfig>, filename: String) -> HttpResponse {
    let mut url = state.public_url.clone();
    url.push('/');
    url.push_str(filename.as_str());

    if accepts_json(req) {
        let mut delete_url = state.public_url.clone();
        delete_url.push_str("/delete/");
        delete_url.push_str(filename.as_str());

        let response = UploadedFile {
            name: filename,
            url,
            delete_url,
        };

        HttpResponse::Created().json(response)
    } else {
        HttpResponse::Created()
            .content_type(ContentType::plaintext())
            .body(url)
    }
}

fn accepts_json(req: &HttpRequest) -> bool {
    match req.headers().get(header::ACCEPT) {
        Some(accept) => accept == mime::APPLICATION_JSON.as_ref(),
        None => false,
    }
}

pub async fn save_file(mut field: Field, upload_dir: &Path) -> Result<String, AppError> {
    let content_disposition = field.content_disposition();
    let filename = content_disposition.get_filename();

    info!("Received file '{}'", filename.unwrap_or("<unk>"));

    let (filename, mut file) = create_file(upload_dir, filename).await?;

    while let Some(chunk) = field.next().await {
        let bytes = chunk?;
        file = web::block(move || file.write_all(&bytes).map(|_| file)).await??;
    }

    debug!("Saved file '{filename}' to media directory");
    Ok(filename)
}

async fn create_file(
    upload_dir: &Path,
    filename: Option<&str>,
) -> Result<(String, std::fs::File), AppError> {
    let upload_dir = upload_dir.to_path_buf(); // created owned copy of the path
    if !upload_dir.exists() {
        debug!(
            "Upload directory '{}' does not exist; creating",
            upload_dir.display()
        );
        std::fs::create_dir_all(&upload_dir)?;
    }

    let filename = match filename {
        Some(name) => sanitize_filename::sanitize(name),
        None => "file".to_string(),
    };

    // Find available filename
    let (filename, file_path) = web::block(move || {
        let file_path = upload_dir.join(filename.as_str());

        // Generate a unique name in case of clash
        if file_path.exists() {
            let salt = format!("{}", Utc::now().format("-%Y%m%dT%H%M"));

            // Separate filename into it's components - name and extension
            let (name, ext) = match filename.rfind('.') {
                Some(index) => {
                    let (n, e) = filename.split_at(index);
                    (n, Some(e))
                }
                None => (filename.as_str(), None),
            };

            // Concat new name
            let mut new_filename = name.to_owned();
            new_filename.push_str(salt.as_str());
            if let Some(ext) = ext {
                new_filename.push_str(ext);
            }

            let new_file_path = upload_dir.join(new_filename.as_str());

            (new_filename, new_file_path)
        } else {
            (filename, file_path)
        }
    })
    .await?;
    debug!("Saving file to: {}", file_path.display());

    let file = web::block(|| std::fs::File::create(file_path)).await??;

    Ok((filename, file))
}

#[get("/purge")]
async fn purge(state: Data<AppConfig>) -> impl Responder {
    let upload_dir = state.upload_dir.clone();
    info!(
        "Purging all files from the upload directory: {}",
        upload_dir.display()
    );

    let result = web::block(move || {
        let entries = std::fs::read_dir(upload_dir.clone());
        match entries {
            Ok(mut entries) => {
                while let Some(Ok(entry)) = entries.next() {
                    let path = entry.path();
                    if path.is_file() {
                        debug!("Purging file: {}", path.display());
                        let res = std::fs::remove_file(path);
                        if res.is_err() {
                            continue;
                        }
                    }
                }

                Ok(())
            }
            Err(error) => Err(error),
        }
    })
    .await;

    if let Err(error) = result {
        return HttpResponse::InternalServerError()
            .content_type(ContentType::plaintext())
            .body(format!("{}", error));
    } else if let Err(error) = result.unwrap() {
        return HttpResponse::InternalServerError()
            .content_type(ContentType::plaintext())
            .body(format!("{}", error));
    }

    HttpResponse::Ok().finish()
}

#[get("/delete/{filename}")]
async fn delete(path: web::Path<String>, state: Data<AppConfig>) -> impl Responder {
    let filename = sanitize_filename::sanitize(path.into_inner());
    let file_path = state.upload_dir.join(filename.as_str());
    info!("Deleting file: {}", file_path.display());

    let result = web::block(|| std::fs::remove_file(file_path)).await;

    if let Err(error) = result {
        return HttpResponse::InternalServerError()
            .content_type(ContentType::plaintext())
            .body(format!("{}", error));
    } else if let Err(error) = result.expect("result to be populated") {
        return HttpResponse::InternalServerError()
            .content_type(ContentType::plaintext())
            .body(format!("{}", error));
    }

    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();
    let cfg: AppConfig = load_config().expect("invalid configuration");

    info!(
        "Listening on http://{}:{}",
        cfg.listen_address, cfg.listen_port
    );

    let cfg_copy = cfg.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(cfg_copy.clone()))
            .wrap(HttpAuthentication::basic(auth::validator))
            .service(upload)
            .service(purge)
            .service(delete)
    })
    .bind((cfg.listen_address.as_ref(), cfg.listen_port))?
    .run()
    .await
}

fn init_log() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

/// Loads the app configurations from a file, or creates one with default values if it doesn't exist.
///
/// On Linux systems, the file can be found on "/home/$USER/.config/mitto/mitto.toml".
fn load_config() -> Result<AppConfig, AppError> {
    let config_path = confy::get_configuration_file_path("mitto", "mitto")?;
    info!("Loading config from '{}'", config_path.display());

    let config = confy::load_path::<AppConfig>(config_path)?;
    info!("Loaded config: {:?}", config);

    Ok(config)
}
