mod cli;

use actix_files::NamedFile;
use actix_web::{
    middleware, rt, web, App, Error as WebError, HttpRequest, HttpResponse, HttpServer, Result,
};
use cat5::hurdat2::{Status, Storm, StormIter};
use cat5::{debug, map, DataDir};
use clap::Parser;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

#[derive(Debug, Clone)]
struct Context {
    data_dir: DataDir,
    dist_dir: PathBuf,
}

impl Context {
    fn new(data_dir: DataDir, dist_dir: PathBuf) -> Context {
        Context { data_dir, dist_dir }
    }
}

fn was_hurricane(storm: &Storm) -> bool {
    storm
        .track()
        .iter()
        .any(|e| e.status() == Status::Hurricane)
}

async fn index(data: web::Data<Context>, req: HttpRequest) -> Result<HttpResponse, WebError> {
    Ok(NamedFile::open("package.json")?
        .disable_content_disposition()
        .into_response(&req))
}

async fn app_main(addr: &str, data: web::Data<Context>) -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/.*").route(web::get().to(index)))
    })
    .bind(addr)?
    .run()
    .await
}

fn start_watcher() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("npm").args(["run", "watch-dev"]).spawn()?;

    ctrlc::set_handler(move || child.kill().expect("unable to kill child"))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let flags = cli::Flags::parse();

    let data_dir = DataDir::at(flags.data_dir())?;
    let mut r = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(data_dir.download_and_open("hurdat2.csv", flags.hurdat2_url())?);
    let hurricanes = StormIter::new(r.records())
        .filter(|r| match r {
            Ok(s) => was_hurricane(s),
            _ => true,
        })
        .collect::<Result<Vec<_>, _>>()?;
    println!("hurricanes: {}", hurricanes.len());

    let m = map::Map::build(
        flags.for_map().svg_file(),
        10.0,
        flags.for_map().land_color(),
    )?;
    debug::render_map(data_dir.join("map.pdf"), &m)?;

    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    if flags.for_web().should_rebuild_assets() {
        start_watcher()?;
    }

    let data = web::Data::new(Context::new(data_dir, flags.dist_dir().to_owned()));
    if let Err(_) = thread::spawn(move || {
        rt::System::new().block_on(app_main(flags.for_web().bind_addr(), data))
    })
    .join()
    {
        return Err("could not wait on server".into());
    }

    Ok(())
}
