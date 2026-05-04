use super::requester::ReqInput;

use tokio::sync::mpsc::Sender;

use rand::seq::SliceRandom;

use reqwest::Url;
// https://static.donationalerts.ru/audiodonations/65473/65473786.wav

pub async fn url_generator(count: usize, req_tx: Sender<ReqInput>, parser_tx: Sender<String>) {
    let mut rng = rand::rng();
    let mut nums_range = (0..10).collect::<Vec<u8>>();

    let _ = req_tx.send(("https://static.donationalerts.ru/audiodonations/65473/65473786.wav".parse().unwrap(), parser_tx.clone())).await;

    for _ in 0..count {
        nums_range.shuffle(&mut rng);
        let id = nums_range.iter()
            .take(8)
            .map(|n| n.to_string())
            .collect::<String>();
        let url = format!("https://static.donationalerts.ru/audiodonations/{}/{}.wav",
            id.get(0..=5).unwrap(), id).parse::<Url>().unwrap();
        let _ = req_tx.send((url, parser_tx.clone())).await;
    }
}
