use std::error::Error;

use csv_async::{AsyncReaderBuilder, StringRecordsStream};
use tokio::io;

use crate::{
    hurdat2::{Status, Storm},
    noaa, DataDir, FetchStrategy,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long, default_value_t = noaa::hurdat2_url().to_owned())]
    hurdat2_url: String,
}

async fn collect_storms<'a, R, F>(
    stream: &mut StringRecordsStream<'a, R>,
    filter: F,
) -> Result<Vec<Storm>, Box<dyn Error>>
where
    R: io::AsyncRead + Unpin + Send,
    F: Fn(&Storm) -> bool,
{
    let mut storms = vec![];
    while let Some(storm) = Storm::from_record_stream(stream).await {
        let storm = storm?;
        if filter(&storm) {
            storms.push(storm);
        }
    }
    Ok(storms)
}

pub async fn run(dir: &DataDir<'_>, args: Args) -> Result<(), Box<dyn Error>> {
    let f = dir
        .get_object("hurdat2.txt")
        .fetch(&args.hurdat2_url, FetchStrategy::Always)
        .await?
        .open()
        .await?;
    let mut stream = AsyncReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .create_reader(f);
    let mut records = stream.records();
    let storms = collect_storms(&mut records, |storm| {
        storm
            .track()
            .iter()
            .any(|e| e.status() == Status::Hurricane)
    })
    .await?;
    println!("{} hurricanes found", storms.len());

    Ok(())
}
