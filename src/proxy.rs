use std::{
    collections::VecDeque,
    net::{SocketAddr, IpAddr},
    sync::Arc, time::Duration,
};

use reqwest::{
    self,
    Proxy, Client
};

use tokio::{
    sync::{mpsc::{self, Sender}, Mutex}, task::JoinSet,
};

const GITHUB_URLS: &[&'static str] = &[
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks5.txt",
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/socks4.txt",
    "https://raw.githubusercontent.com/TheSpeedX/SOCKS-List/master/http.txt",
];

pub async fn fetch_all_proxy() -> Result<VecDeque<SocketAddr>, reqwest::Error> {
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

async fn proxy_checker(proxy_list: Arc<Mutex<VecDeque<SocketAddr>>>, tx: Sender<Option<SocketAddr>>) {
    loop {
        let proxy_to_check = {
            let mut lock = proxy_list.lock().await;
            let Some(proxy) = (*lock).pop_front() else {
                break;
            };
            proxy
        };
        let Ok(proxy_ip) = get_my_ip(Proxy::all(proxy_to_check.to_string()).unwrap()).await else {
            continue;
        };
        log::trace!("Checked");
        if proxy_ip == proxy_to_check.ip() {
            continue;
        }
        // TODO: Handle this mfc
        let _ = tx.send(Some(proxy_to_check)).await;
    }
}

pub async fn check_proxies(proxies: VecDeque<SocketAddr>, jobs: usize) -> Vec<SocketAddr> {
    let proxies = Arc::new(Mutex::new(proxies));
    let (tx, mut rx) = mpsc::channel(100);

    let mut workers_pool = JoinSet::new();
    for _ in 0..jobs {
        let proxies = proxies.clone();
        let tx = tx.clone();
        workers_pool.spawn(async move {
            proxy_checker(proxies, tx).await
        });
    }

    let working_proxies = tokio::spawn(async move {
        let mut working_proxies = vec![];
        while let Some(ip) = rx.recv().await {
            log::error!("Found working IP");
            working_proxies.push(ip);
        }
        working_proxies
    });

    let res = tokio::join!(
        working_proxies,
        workers_pool.join_all(),
    );

    dbg!(res);

    unimplemented!();
}

pub async fn get_my_ip(proxy: Proxy) -> Result<IpAddr, reqwest::Error> {
    let client = Client::builder()
        .connect_timeout(Duration::new(5, 0))
        .proxy(proxy.clone())
        .build().unwrap();
    let ip = client.get("https://api.ipify.org").send().await?.text().await?;
    Ok(ip.parse().unwrap())
}
