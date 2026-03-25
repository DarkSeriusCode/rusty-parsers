use log::{LevelFilter};

mod proxy;
mod parser;
mod parsers;
mod utils;

#[tokio::main]
async fn main() {
    colog::basic_builder()
        .filter(None, LevelFilter::Off)
        .filter(Some(env!("CARGO_CRATE_NAME")), LevelFilter::Trace)
        .init();
    log::info!("Fetching proxies");
    let Ok(all_proxies) = proxy::fetch_all_proxy().await else {
        log::error!("Unable to fetch proxies!");
        return;
    };
    log::info!("Fetched {} proxies", all_proxies.len());

    dbg!(proxy::check_proxies(all_proxies.clone(), all_proxies.len()).await);
}
