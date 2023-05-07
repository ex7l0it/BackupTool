use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
#[macro_use]
extern crate log;
mod tools;
use tools::*;

#[derive(Parser, Debug)]
#[command(author = "Ex7l0it")]
struct Opts {
    /// Path to config file
    #[arg(short = 'c', long, default_value = "./config.yaml")]
    config: String,
    /// Path to backup file
    #[arg(short = 'o', long, default_value = "./bkup/")]
    output: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Task {
    name: String,
    #[serde(deserialize_with = "parse_mypathbuf", rename="path")]
    srcpath: Vec<MyPathBuf>,
    #[serde(default)]
    dstpath: Option<MyPathBuf>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MyPathBuf {
    path_str: String,
    #[serde(skip)]
    path: PathBuf,
}

impl FromStr for MyPathBuf {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = PathBuf::from(s);
        if s.starts_with("~/") {
            // Get home directory from std::env
            let home_dir = std::env::var("HOME").unwrap();
            path = PathBuf::from(s.replace('~', &home_dir));
        }
        Ok(Self {
            path_str: s.to_string(),
            path,
        })
    }
}

impl std::fmt::Display for MyPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.path_str)
    }
}

impl MyPathBuf {
    fn exists(&self) -> bool {
        self.path.exists()
    }
}

impl Task {
    // backup processing
    fn process_backup(&self, dstpath: &Path) -> Result<()> {
        // create directory
        let dstpath = dstpath.join(&self.name);
        if !dstpath.exists() {
            warn!("creating task directory: {:?}", dstpath);
            std::fs::create_dir_all(&dstpath)?;
        }
        // check if path exists
        for srcpath in &self.srcpath {
            if !srcpath.exists() {
                warn!("Path does not exist: {}", srcpath);
                continue;
            }
            // copy file or directory
            let dstpath = dstpath.join(srcpath.path.file_name().unwrap());
            std::fs::copy(&srcpath.path, dstpath).map_err(|e| anyhow!("Copy ERROR: {e}"))?;
        }

        Ok(())
    }
}

fn parse_mypathbuf<'de, D>(deserializer: D) -> Result<Vec<MyPathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Vec<String> = serde::Deserialize::deserialize(deserializer)?;
    let mut v = Vec::new();
    for i in s {
        v.push(i.parse().unwrap());
    }
    Ok(v)
}

fn parse_config(path: &str) -> Result<Vec<Task>> {
    let file =
        std::fs::File::open(path).map_err(|e| anyhow!("Open config file Failed: {path} - {e}"))?;

    // parse .yaml file
    let configs: Vec<Task> = serde_yaml::from_reader(file)?;

    debug!("{:#?}", configs);
    Ok(configs)
}

fn process_backups(tasks: Vec<Task>, opts: &Opts) -> Result<()> {
    let dstpath = MyPathBuf::from_str(&opts.output)?;
    if !dstpath.exists() {
        warn!("creating output directory: {}", dstpath);
        std::fs::create_dir_all(&dstpath.path)?;
    }
    let temp_dir =
        generate_tempdir(&dstpath.path).map_err(|e| anyhow!("Make Tempdir ERROR: {e}"))?;

    for task in tasks {
        task.process_backup(&temp_dir)?;
    }
    // copy config file to temp directory
    let config_path = PathBuf::from(&opts.config);
    std::fs::copy(
        &config_path,
        temp_dir.join(config_path.file_name().unwrap()),
    )
    .map_err(|e| anyhow!("Copy ERROR: {e}"))?;

    // compress directory
    let tarfile = compress_tar_gz_target(&temp_dir, &dstpath.path)?;
    // remove temp directory
    std::fs::remove_dir_all(&temp_dir).map_err(|e| anyhow!("Remove ERROR: {e}"))?;

    println!("Backup complete: {tarfile:?}");

    Ok(())
}

pub fn run() -> Result<()> {
    let opts = Opts::parse();
    debug!("{:#?}", opts);
    let tasks = parse_config(&opts.config)?;

    process_backups(tasks, &opts)?;

    Ok(())
}
