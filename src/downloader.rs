use ::rayon::prelude::*;
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "crab-dl")]
#[command(about = "lightweight, multithreaded file downloader")]
struct Args {
    #[arg(value_name = "DIRECTORY", required = true)]
    download_dir: PathBuf,

    #[arg(value_name = "URL", required = true)]
    urls: Vec<String>,
}

fn download_file(
    client: &Client,
    url: &str,
    path: &str,
    mp: &MultiProgress,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut resp = client.get(url).send()?;
    let mut file = File::create(path)?;

    let total_size = resp
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    let pb = mp.add(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("=>-"));

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = resp.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    Ok(())
}

fn format_filename(download_dir: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let entries = fs::read_dir(download_dir)?;
    let mut existing_files: Vec<String> = vec![];

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        let filename: String = path_str.chars().skip(download_dir.len()).collect();
        let parsed_entry = filename.split(".").next().unwrap().to_string();
        existing_files.push(parsed_entry);
    }

    let mut passes = 0;
    let mut count = 1;
    loop {
        let formatted_filename = format!("download_{:02}", count);
        if existing_files.contains(&formatted_filename) {
            count += 1;
        }
        passes += 1;

        if passes > existing_files.len() {
            return Ok(count);
        }
    }
}

fn get_extension(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = url.split('?').next().unwrap_or(url);
    let last_segment = path.split('/').last().unwrap_or("");

    if let Some(pos) = last_segment.rfind('.') {
        if last_segment.contains(".tar.") {
            return Ok(format!(".tar{}", last_segment[pos..].to_string()));
        }
        return Ok(last_segment[pos..].to_string());
    }

    Ok(".bin".to_string())
}

pub fn initiate_download(
    download_dir: &str,
    urls: &Vec<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let download_tasks: Vec<_> = urls
        .iter()
        .map(|url| {
            let filename = format_filename(download_dir)?;
            let extension = get_extension(url)?;
            let path = format!("{}download_{:02}{}", download_dir, filename, extension);
            File::create(&path).ok();
            Ok((url.to_string(), path))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    let mp = MultiProgress::new();

    download_tasks
        .par_iter()
        .enumerate()
        .for_each(|(idx, (url, path))| {
            let client = Client::new();
            match download_file(&client, url, path, &mp) {
                Ok(_) => println!("Downloaded file {}", idx + 1),
                Err(e) => println!("Download {} failed: {}", idx + 1, e),
            }
        });

    Ok(())
}

pub fn run_cli() {
    let args = Args::parse();
    let download_dir = args.download_dir.into_os_string();
    let mut urls = vec![];

    for url in &args.urls {
        urls.push(url);
    }

    if let Some(str) = download_dir.to_str() {
        match initiate_download(str, &urls) {
            Ok(_) => {
                println!("Downloads completed!");
            }
            Err(e) => {
                println!(
                    "Download failed: {}\nUsage: crab-dl <DIRECTORY> <URL>...",
                    e
                );
            }
        };
    }
}
