use std::fs;
use std::fs::File;
use std::io::Read;
use notify::{Watcher, RecursiveMode, Result, Event};
use std::path::{Path, PathBuf};
use std::process::Command;
use structopt::StructOpt;
use reqwest::Client;
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
    #[structopt(short = "l", long = "url", default_value = "http://127.0.0.1:8090")]
    url: String,

    /// The username to the qbittorrent server
    #[structopt(short = "u", long = "username")]
    username: String,

    /// The password to the qbittorrent server
    #[structopt(short = "p", long = "password")]
    password: String,
}

#[tokio::main]
async fn main() -> Result<(), > {
    let opt = Opt::from_args();
    let out = opt.output.clone();
    let url = opt.url.clone();
    let password = opt.password.clone();
    let username = opt.username.clone();
    let out2 = opt.output.clone();
    tokio::spawn(async move {
        let entries = fs::read_dir(out).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            println!("{:?}", path);
            if path.is_file() {
                send_torrent(&path, &url, &password, &username).await;
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
        .arg("-o")
        .arg(full_output_path)
        .arg("-i")
        .arg(path)
        .output().unwrap();
    if !output.status.success() {
        eprintln!("failed: {}", String::from_utf8_lossy(&output.stderr));
    }
}

async fn send_torrent(full_output_path: &PathBuf, url: &String, x: &String, x0: &String) {
    let client = Client::new();
    let torrent_data = fs::read(full_output_path).unwrap();
    let torrent_path = full_output_path.file_name().unwrap().to_str().unwrap();
    let save_path = "/downloads";
    let url = format!("{}/api/v2/", url);

    let mut file = File::open(torrent_path).unwrap();
    let mut torrent_data = Vec::new();
    file.read_to_end(&mut torrent_data).unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "id":  1,
        "method": "torrents/add",
        "params": {
            "urls": [base64::encode(&torrent_data)],
            "save_path": "/path/to/save/location", // Replace with the path where you want the torrent to be saved
            "auto_tmm": false,
            "sequential_download": false,
            "first_last_piece_priority": false
        }
    });

    let response = client.post(url)
        .basic_auth(x, Some(x0))
        .json(&request_body)
        .send()
        .await
        .unwrap();

    println!("{:?}", response);
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