use std::error::Error;

use csv_async::AsyncReaderBuilder;
use csv_async::StringRecordsStream;
use tokio::{fs, io};

use crate::hurdat2::{Status, Storm};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(help = "CSV file containing hurdat2 data")]
    src: String,

    #[clap(help = "path to where the JSON output file should be written")]
    dst: String,
}

pub async fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let r = fs::File::open(&args.src).await?;

    let mut stream = AsyncReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .create_reader(r);
    let mut records = stream.records();
    let storms = collect_storms(&mut records, |s| {
        s.track().iter().any(|e| e.status() == Status::Hurricane)
    })
    .await?;

    let json = serde_json::to_string(&storms)?;

    fs::write(&args.dst, json).await?;

    Ok(())
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
