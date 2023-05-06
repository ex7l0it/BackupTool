use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
#[macro_use]
extern crate log;
mod tools;
use tools::*;

#[derive(Parser, Debug)]
struct Opts {
    /// Path to config file
    #[arg(short = 'c', long, default_value = "./config.yaml")]
    config: String,
    /// Path to backup file
    #[arg(short = 'o', long, default_value = "./bkup/")]
    output: String,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Task {
    name: String,
    #[serde(deserialize_with = "parse_mypathbuf")]
    srcpath: MyPathBuf,
    #[serde(default)]
    dstpath: Option<MyPathBuf>,
    group: String,
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
            path = PathBuf::from(s.replace("~", &home_dir));
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
    fn process(&self, opts: &Opts) -> Result<()> {
        // check if path exists
        if !self.srcpath.exists() {
            warn!("Path does not exist: {}", self.srcpath);
            // Pass
            return Ok(());
        }
        // check if dstpath exists
        let dstpath = MyPathBuf::from_str(&opts.output)?;
        if !dstpath.exists() {
            warn!("creating output directory: {}", dstpath);
            std::fs::create_dir_all(&dstpath.path)?;
        }
        // copy file or directory
        let srcpath = &self.srcpath.path;
        let dstpath = dstpath.path.join(srcpath.file_name().unwrap());
        std::fs::copy(srcpath, dstpath)?;

        Ok(())
    }
}

fn parse_mypathbuf<'de, D>(deserializer: D) -> Result<MyPathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
}

fn parse_config(path: &str) -> Result<Vec<Task>> {
    let file = std::fs::File::open(path)?;

    // parse .yaml file
    let configs: Vec<Task> = serde_yaml::from_reader(file)?;

    debug!("{:#?}", configs);
    Ok(configs)
}

pub fn run() -> Result<()> {
    let opts = Opts::parse();
    debug!("{:#?}", opts);
    let tasks = parse_config(&opts.config)?;
    for task in tasks {
        task.process(&opts)?;
    }

    Ok(())
}
