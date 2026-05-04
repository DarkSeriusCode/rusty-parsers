use log::{LevelFilter};
use std::{
    time::Duration,
    fs, io::{Read, Write},
    net::SocketAddr,
    collections::VecDeque,
};
use clap::{Args, Parser, Subcommand};

mod proxy;

// async fn proxy_huyoksy() {
//     let unchecked_proxies = proxy::fetch_proxies().await.unwrap();
//     log::info!("{}", unchecked_proxies.len());
//     let res = proxy::check_proxies(unchecked_proxies, 1<<9).await;

//     log::info!("Got {}", res.len());

//     let mut fs = std::fs::File::create("proxies.txt").unwrap();
//     for add in res {
//         write!(fs, "{}\n", add.to_string());
//     }
// }

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    FindProxy(FindProxyArgs)
}

#[derive(Args)]
struct FindProxyArgs {
    #[arg(short, long, default_value_t=String::from("proxies.txt"))]
    output: String,
    #[arg(short, long)]
    from_file: Option<String>,
    #[arg(short, long, default_value_t=15)]
    timeout: u64,
    #[arg(short, long, default_value_t=32)]
    jobs: usize,
}

#[tokio::main]
async fn main() {
    colog::basic_builder()
        .filter(None, LevelFilter::Off)
        .filter(Some(env!("CARGO_CRATE_NAME")), LevelFilter::Trace)
        .init();

    let cli = Cli::parse();

    let res = match cli.command {
        Commands::FindProxy(args) => find_proxy(args).await,
    };

    if let Err(err) = res {
        log::error!("ERROR: {err}");
    }
}

async fn find_proxy(args: FindProxyArgs) -> anyhow::Result<()> {
    let proxies_to_check = match args.from_file {
        Some(file_path) => {
            log::info!("Getting proxies from the file {}", file_path);
            let mut file = fs::File::open(file_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            content.split_whitespace().map(|s| s.parse().unwrap()).collect::<VecDeque<SocketAddr>>()

        }
        None => {
            log::info!("Getting proxies from the web");
            proxy::fetch_proxies().await?
        }
    };
    let proxies_to_check_count = proxies_to_check.len();
    log::info!("Got proxies: {}", proxies_to_check_count);

    log::info!("Start scanning:\nTimeout: {}s\nJobs: {}", args.timeout, args.jobs);

    let working_proxies = proxy::check_proxies(proxies_to_check,
                                               args.jobs,
                                               Duration::from_secs(args.timeout)
                                               ).await;

    log::info!("Found {0} proxies ({0}/{1})", working_proxies.len(), proxies_to_check_count);

    let mut fs = fs::File::create(args.output)?;
    for proxy in working_proxies {
        write!(fs, "{}\n", proxy.to_string())?;
    }

    Ok(())
}
