use std::{fs, io};
use reqwest::Url;

pub struct UrlGen {
    last_checked_url: Url,
}

impl UrlGen {
    const CACHE_FILE: &'static str = "cache/ptrscr_urls.txt";

    pub fn new() -> io::Result<Self> {
        unimplemented!()
    }
}
