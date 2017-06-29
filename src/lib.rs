extern crate hyper;
extern crate hyper_native_tls;
extern crate pbr;
extern crate regex;

use pbr::ProgressBar;
use std::{process,str};
use std::collections::HashMap;
use hyper::client::response::Response;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::header::ContentLength;
use std::io::Read;
use std::io::prelude::*;
use std::fs::File;
use regex::Regex;

struct Info {
    video_url: String,
}

pub fn new(video_url: &str) -> Info {
    //Regex for youtube URLs.
    let url_regex = Regex::new(r"^.*(?:(?:youtu\.be/|v/|vi/|u/w/|embed/)|(?:(?:watch)?\?v(?:i)?=|\&v(?:i)?=))([^#\&\?]*).*").unwrap();
    let mut vid = "https://www.youtube.com/watch?v=DjMkfARvGE8";
    if url_regex.is_match(vid) {
        let vid_split = url_regex.captures(vid).unwrap();
        vid = vid_split.get(1).unwrap().as_str();
    }
    let url = format!("https://youtube.com/get_video_info?video_id={}", vid);
    download(&url);
    Info { video_url : video_url }
}

fn download(url: &str) {
    let mut response = send_request(url);
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();
    let hq = parse_url(&response_str);

    if hq["status"] != "ok" {
        println!("Video not found!");
        process::exit(1);
    }

    // get video info
    let streams: Vec<&str> = hq["url_encoded_fmt_stream_map"]
        .split(',')
        .collect();

    // list of available qualities
    let mut qualities: HashMap<i32, (String, String)> = HashMap::new();
    for (i, url) in streams.iter().enumerate() {
        let quality = parse_url(url);
        let extension = quality["type"]
            .split('/')
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        qualities.insert(i as i32,
                         (quality["url"].to_string(), extension.to_owned()));
        println!("{}- {} {}",
                 i,
                 quality["quality"],
                 quality["type"]);
    }
}

fn send_request(url: &str) -> Response {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    client.get(url).send().unwrap_or_else(|e| {
        println!("Network request failed: {}", e);
        process::exit(1);
    })
}

fn parse_url(query: &str) -> HashMap<String, String> {
    let u = format!("{}{}", "http://e.com?", query);
    let parsed_url = hyper::Url::parse(&u).unwrap();
    parsed_url.query_pairs().into_owned().collect()
}
