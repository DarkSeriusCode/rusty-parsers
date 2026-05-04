use std::{
    collections::VecDeque,
    net::{SocketAddr, IpAddr},
    sync::Arc, time::Duration,
    error::Error,
};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    self,
    Proxy, Client
};
use tokio::{
    sync::Mutex, task::JoinSet,
};

const GITHUB_URLS: &[&'static str] = &[
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks4.txt",
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/http.txt",
];

pub async fn fetch_proxies() -> Result<VecDeque<SocketAddr>, reqwest::Error> {
    let client = Client::new();
    let mut proxies = VecDeque::new();

    for url in GITHUB_URLS {
        let content = client.get(*url).send().await?.text().await?;
        for proxy_url in content.split_whitespace() {
            proxies.push_back(proxy_url.parse().unwrap());
        }
    }

    Ok(proxies)
}

async fn proxy_checker(proxy_list: Arc<Mutex<VecDeque<SocketAddr>>>,
                       timeout: Duration, progress_bar: ProgressBar) -> Vec<SocketAddr>
{
    let mut working_proxies = vec![];
    loop {
        let proxy_to_check = {
            let mut lock = proxy_list.lock().await;
            let Some(proxy) = (*lock).pop_front() else {
                break;
            };
            proxy
        };
        let ok_ip = get_my_ip(proxy_to_check, timeout).await;
        progress_bar.inc(1);
        let proxy_ip = match ok_ip {
            Ok(ip) => ip,
            Err(err) => {
                // progress_bar.suspend(|| { log::warn!("{}", err.source().unwrap().to_string()); });
                continue;
            }
        };
        if proxy_ip == proxy_to_check.ip() {
            continue;
        }
        progress_bar.suspend(|| {
            log::info!("Found working proxy");
        });
        working_proxies.push(proxy_to_check);
    }
    working_proxies
}

pub async fn check_proxies(proxies: VecDeque<SocketAddr>, jobs: usize,
                           timeout: Duration) -> Vec<SocketAddr>
{
    let pb = ProgressBar::new(proxies.len().try_into().unwrap());
    pb.enable_steady_tick(Duration::from_millis(60));
    pb.set_style(
            ProgressStyle::with_template("[{spinner:.green.bold}] |{bar:50}|  {pos}/{len} ({percent}%)")
            .unwrap().progress_chars("=> "),
    );
    let proxies = Arc::new(Mutex::new(proxies));

    let mut workers_pool = JoinSet::new();
    for _ in 0..jobs {
        let proxies = proxies.clone();
        let pb = pb.clone();
        workers_pool.spawn(async move {
            proxy_checker(proxies, timeout, pb).await
        });
    }

    let result = workers_pool.join_all().await
        .iter().flatten()
        .map(|s| s.to_owned())
        .collect::<Vec<SocketAddr>>();
    pb.finish_and_clear();
    result
}

pub async fn get_my_ip(proxy_addr: SocketAddr, timeout: Duration) -> Result<IpAddr, reqwest::Error> {
    let client = Client::builder()
        .connect_timeout(timeout)
        .timeout(timeout)
        .proxy(Proxy::all(proxy_addr.to_string())?)
        .build().unwrap();
    let ip = client.get("https://api.ipify.org").send().await?.text().await?;
    Ok(ip.parse().unwrap())
}
