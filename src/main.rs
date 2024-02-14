use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use notify::{Watcher, RecursiveMode, Result, Event};
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::StructOpt;
use reqwest::{Client, header, multipart};
use serde_json::json;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time::sleep;

// Define the command line arguments
#[derive(Debug, StructOpt)]
#[structopt(name = "torrent-creator")]
struct Opt {
    /// The folder to monitor for changes
    #[structopt(short = "f", long = "folder")]
    folder: String,

    /// The output folder where the torrent files will be saved
    #[structopt(short = "o", long = "output")]
    output: String,

    /// The url to the qbittorrent server. default is http://127.0.0.1:8090
    #[structopt(short = "u", long = "url", default_value = "http://127.0.0.1:8090")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), > {
    let opt = Opt::from_args();
    let out = opt.output.clone();
    let url = opt.url.clone();
    tokio::spawn(async move {
        let entries = fs::read_dir(out).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            println!("{:?}", path);
            if path.is_file() {
                send_torrent(&path, &url).await;
            }
            println!("done")
        }
    });

    let callback = move |result: Result<Event>| {
        let out = opt.output.clone();
        handle(out, result);
    };
    // Parse command line arguments

    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(callback)?;

    watcher.watch(Path::new(&opt.folder), RecursiveMode::NonRecursive)?;

    loop {
    }
}

fn torrent(full_output_path: &PathBuf, path: PathBuf) {
    let output = Command::new("imdl")
        .arg("torrent")
        .arg("create")
        .arg("-c Created using auto_torrent in rust by valgul.")
        .arg("-o")
        .arg(full_output_path)
        .arg("-i")
        .arg(path)
        .output().unwrap();
    if !output.status.success() {
        eprintln!("failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}

async fn send_torrent(full_output_path: &PathBuf, url: &String) {
    let url = format!("{}/api/v2/torrents/add", url);

   let output  = Command::new("imdl")
        .arg("torrent")
        .arg("create")
        .arg("--link")
        .arg(full_output_path)
        .output().unwrap();

    let magnet_url = String::from_utf8(output.stdout).unwrap();
    println!("Magnet URL: {}", magnet_url);();

    let form = reqwest::multipart::Form::new()
        .text("root_folder", "true")
        .text("urls", magnet_url);



    let client = Client::new();
    let response = client.post(url)
        .header(header::CONTENT_TYPE, "multipart/form-data")
        .header(header::REFERER, "https://qbittorrent.valgul.moe/")
        .header(header::ORIGIN, "https://qbittorrent.valgul.moe/")
        .multipart(form)
        .send().await.unwrap();


    println!("Response: {:?}", response);
}

fn handle(out: String, result: Result<Event>) {
    match result {
        Ok(event) => {
            // Check if the event kind is a modification event
            if event.kind.is_modify() | event.kind.is_create() {
                println!("File: {:?}", event.paths[0]);
                let path = event.paths[0].clone();
                let last_part = path.file_name().and_then(|os_str| os_str.to_str()).unwrap_or_default();
                let output_dir = out.clone();
                let torrent_filename = format!("{}.torrent", last_part);
                let full_output_path = Path::new(&output_dir).join(&torrent_filename);
                let mut version =  1;
                let mut full_output_path_with_version = full_output_path.clone();
                while full_output_path_with_version.exists() {
                    let old_output_path = Path::new(&output_dir).join("old").join(format!("v{}", version));
                    fs::create_dir_all(&old_output_path).unwrap();
                    fs::rename(&full_output_path_with_version, old_output_path.join(&torrent_filename)).unwrap();
                    version +=  1;
                    full_output_path_with_version = Path::new(&out).join(format!("old/v{}/{}", version, &torrent_filename));
                }
                torrent(&full_output_path, path);
                println!("done")
            } else if event.kind.is_remove() {
                println!("File removed: {:?}", event.paths[0]);
            } else {
                println!("Other event: {:?}", event.kind);
            }
        },
        Err(e) => println!("watch error: {:?}", e),
    };
}