use super::DataDir;
use actix_files::NamedFile;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Context {
	data_dir: DataDir,
	dist_dir: PathBuf,
}

impl Context {
	pub fn new(data_dir: DataDir, dist_dir: PathBuf) -> Context {
		Context { data_dir, dist_dir }
	}

	fn asset_for(&self, req: &HttpRequest) -> PathBuf {
		let path = req.path();
		let path = if path.starts_with("/data/") {
			self.data_dir.join(&path[6..])
		} else {
			self.dist_dir.join(&path[1..])
		};
		if path.is_dir() {
			path.join("index.html")
		} else {
			path
		}
	}
}

async fn index(
	data: web::Data<Context>,
	req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
	let path = data.asset_for(&req);
	Ok(NamedFile::open(path)?
		.disable_content_disposition()
		.into_response(&req))
}

pub async fn run_app(addr: &str, data: web::Data<Context>) -> io::Result<()> {
	HttpServer::new(move || {
		App::new()
			.app_data(data.clone())
			.wrap(middleware::Logger::default())
			.default_service(web::get().to(index))
	})
	.bind(addr)?
	.run()
	.await
}
