#![windows_subsystem = "windows"]
mod utils;
use clap::{Arg, Command};
use futures_util::stream::{FuturesUnordered, StreamExt};
use glob::glob;
use home_config::HomeConfig;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs::write;
use std::env;
use tinypng::{TinyPng, REGISTER_URL};
use utils::format_size;
use win_toast_notify::*;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    key: String,
}

#[tokio::main]
async fn main() {
    // Adding the logo resources to a user's temporary folder
    let logo_path = env::temp_dir().join("tinypng.png");
    let icon_data = include_bytes!("../resources/tinypng.png");

    match logo_path.try_exists() {
        Ok(true) => {
            if !logo_path.is_file() {
                write(&logo_path, icon_data).expect("Unable to write file");
            }
        },
        Ok(false) => {
            write(&logo_path, icon_data).expect("Unable to write file");
        },
        Err(err) => {
            WinToastNotify::new()
                .set_title(&format!("Failed to run task {:?}", err))
                .set_logo(logo_path.to_str().expect("The icon path is an invalid Unicode"), CropCircle::True)
                .show()
                .expect("Failed to show toast notification");
            eprintln!("Failed to check if path exists: {:?}", err);
        },
    };

    let app = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("key")
                .short('k')
                .takes_value(true)
                .value_name("API_KEY")
                .help("Set TinyPNG API KEY"),
        )
        .arg(
            Arg::new("image")
                .conflicts_with("key")
                .required(true)
                .min_values(1)
                .help("Images to be compressed"),
        )
        .get_matches();

    let hc = HomeConfig::with_config_dir(env!("CARGO_PKG_NAME"), "config.toml");
    let mut config = hc.toml::<Config>().unwrap_or_default();

    // Set API KEY
    if let Some(key) = app.value_of("key") {
        if key.len() != 32 {
            exit!("Invalid API KEY");
        }
        config.key = key.to_string();
        hc.save_toml(&config).unwrap_or_else(|err| {
            exit!("{:#?}", err);
        });

        WinToastNotify::new()
            .set_title("Successfully set the API KEY for TinyPNG")
            .set_logo(logo_path.to_str().expect("The icon path is an invalid Unicode"), CropCircle::True)
            .show()
            .expect("Failed to show toast notification");
        
        return;
    }

    if config.key.len() != 32 {
        exit!("Invalid API KEY
            \n1. Register a KEY using your email at {REGISTER_URL}
            \n2. Use 'tinypng -k <KEY>' to set API_KEY");
    }

    let tiny = Arc::new(TinyPng::new(config.key));

    let mut paths = app
        .values_of("image")
        .unwrap()
        .flat_map(|val| {
            glob(val).unwrap_or_else(|err| {
                exit!("{:#?}", err);
            })
        })
        .filter_map(|rst| match rst {
            Ok(p) => {
                if p.is_file() {
                    Some(p)
                } else {
                    None
                }
            }
            Err(err) => {
                exit!("{:#?}", err)
            }
        })
        .collect::<VecDeque<PathBuf>>();

    let mut fus = FuturesUnordered::new();

    let task = |tiny: Arc<TinyPng>, p: PathBuf| {
        tokio::spawn(async move { (tiny.compress_file(&p, &p).await, p) })
    };

    // Maximum number of tasks to run simultaneously
    let n = paths.len().min(8);

    for _ in 0..n {
        let p = paths.pop_front().unwrap();
        fus.push(task(tiny.clone(), p));
    }

    while let Some(rst) = fus.next().await {
        if let Some(p) = paths.pop_front() {
            fus.push(task(tiny.clone(), p));
        }
        match rst {
            Ok((rst, p)) => match rst {
                Ok((input, output)) => {
                    let path_string = p.to_string_lossy().into_owned();
                    let ratio = (1.0 - (output as f32 / input as f32)) * 100.0;
                    let (input, output) = (format_size(input), format_size(output));
                    let emojis = [
                        (80.0, "🥰"),
                        (60.0, "🥳"),
                        (40.0, "😋"),
                        (30.0, "😚"),
                        (20.0, "🙂"),
                        (10.0, "😧"),
                        (5.0, "😨"),
                        (1.0, "🤡"),
                    ];
                    let variation = emojis.iter()
                        .find(|&&(threshold, _)| ratio > threshold)
                        .map(|&(_, emoji)| format!("{} ⇒ {} ({:.1}%) {}", input, output, ratio, emoji))
                        .unwrap_or_else(|| format!("{} ⇒ {} ({:.1}%) 🤡", input, output, ratio));

                    WinToastNotify::new()
                        .set_open(&path_string)
                        .set_title("Compress by TinyPNG")
                        .set_messages(vec![
                            &path_string,
                            &variation,
                        ])
                        .set_image(&path_string, ImagePlacement::Top)
                        .set_logo(logo_path.to_str().expect("The icon path is an invalid Unicode"), CropCircle::True)
                        .set_audio(Audio::WinLoopingAlarm5, Loop::False)
                        .show()
                        .expect("Failed to show toast notification")
                }
                Err(err) => {
                    WinToastNotify::new()
                        .set_title(&format!("{}: {:?}", p.display(), err))
                        .set_logo(logo_path.to_str().expect("The icon path is an invalid Unicode"), CropCircle::True)
                        .show()
                        .expect("Failed to show toast notification");
                    eprintln!("{}: {:?}", p.display(), err);
                }
            },
            Err(err) => {
                WinToastNotify::new()
                    .set_title(&format!("Failed to run task {:?}", err))
                    .set_logo(logo_path.to_str().expect("The icon path is an invalid Unicode"), CropCircle::True)
                    .show()
                    .expect("Failed to show toast notification");
                eprintln!("Failed to run task {:?}", err);
            }
        }
    }
}