mod cli;

use actix_web::{rt, web};
use cat5::hurdat2::{Status, Storm, StormIter};
use cat5::{debug, geo, map, DataDir};
use clap::Parser;
use std::error::Error;
use std::io::BufWriter;
use std::process::Command;
use std::thread;

fn was_hurricane(storm: &Storm) -> bool {
    storm
        .track()
        .iter()
        .any(|e| e.status() == Status::Hurricane)
}

fn start_watcher() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("npm").args(["run", "watch-dev"]).spawn()?;

    // terminate the child on signals
    ctrlc::set_handler(move || child.kill().expect("unable to kill watcher subprocess"))?;

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
        geo::Mercator::new(
            10368.61626248217,
            10310.9627199,
            -2160.1283880171186,
            -3566.7693291,
        ),
    )?;

    debug::render_map(data_dir.join("map.pdf"), &m)?;
    debug::export::storms(BufWriter::new(data_dir.create("storms.json")?), &hurricanes)?;
    debug::export::map(BufWriter::new(data_dir.create("map.json")?), &m)?;

    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    if flags.for_web().should_rebuild_assets() {
        start_watcher()?;
    }

    let data = web::Data::new(cat5::web::Context::new(
        data_dir,
        flags.dist_dir().to_owned(),
    ));
    if let Err(_) = thread::spawn(move || {
        rt::System::new().block_on(cat5::web::run_app(flags.for_web().bind_addr(), data))
    })
    .join()
    {
        return Err("could not wait on server".into());
    }

    Ok(())
}
