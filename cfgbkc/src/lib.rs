use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use prettytable::Table;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};
#[macro_use]
extern crate log;
#[macro_use]
extern crate prettytable;
mod tools;
use tools::*;

#[derive(Parser, Debug)]
#[command(author = "Ex7l0it")]
struct Opts {
    /// Path to config file
    #[arg(short = 'c', long, default_value = "./config.yaml", group = "m")]
    config: String,
    /// Path to backup file
    #[arg(short = 'o', long, default_value = "./bkup/")]
    output: String,
    /// Custom Archive File Name
    #[arg(short = 'n', long)]
    name: Option<String>,
    /// Path to backup file
    #[arg(short = 'f', long, group = "m")]
    bkfile: Option<String>,
    /// Mode of operation
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    Backup,
    Restore,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Task {
    name: String,
    #[serde(deserialize_with = "parse_mypathbuf", rename = "path")]
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
    fn backup(&self, dstpath: &Path) -> Result<()> {
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

    // restore processing
    fn restore(&self, backup_dir: &Path, decompress_path: &Path) -> Result<()> {
        // backup old config files
        self.backup(backup_dir)?;
        // restore files
        let group_name = &self.name;
        for srcpath in &self.srcpath {
            let file_path = decompress_path
                .join(group_name)
                .join(srcpath.path.file_name().unwrap());
            // copy
            match std::fs::copy(&file_path, &srcpath.path) {
                Ok(_) => info!("Restore file: {:?}", &srcpath.path),
                Err(e) => warn!("Restore file failed: {:?} - {}", &srcpath.path, e),
            }
        }
        Ok(())
    }
}

trait TasksInfo {
    fn dump_tasks(&self, path: &Path);
}

impl TasksInfo for Vec<Task> {
    fn dump_tasks(&self, path: &Path) {
        let mut table = Table::new();
        table.add_row(row!["GroupName", "FileName", "RestorePath", "Status"]);
        for task in self {
            let group_name = &task.name;
            for srcpath in &task.srcpath {
                let file_name = srcpath.path.file_name().unwrap().to_str().unwrap();
                let check_target_path = path.join(group_name).join(file_name);
                let status = if check_target_path.exists() {
                    "OK"
                } else {
                    "NG"
                };
                table.add_row(row![group_name, file_name, srcpath, status]);
            }
        }
        table.printstd();
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

    // debug!("{:#?}", configs);
    Ok(configs)
}

fn process_backups(tasks: Vec<Task>, opts: &Opts) -> Result<()> {
    let dstpath = MyPathBuf::from_str(&opts.output)?;
    if !dstpath.exists() {
        warn!("creating output directory: {}", dstpath);
        std::fs::create_dir_all(&dstpath.path)?;
    }
    let temp_dir = generate_tempdir(&PathBuf::from("/tmp"))?;

    for task in tasks {
        task.backup(&temp_dir)?;
    }
    // copy config file to temp directory
    let config_path = PathBuf::from(&opts.config);
    std::fs::copy(
        &config_path,
        temp_dir.join(config_path.file_name().unwrap()),
    )
    .map_err(|e| anyhow!("Copy ERROR: {e}"))?;

    // compress directory
    let mut tar_filename = String::from("backup_");
    if let Some(custom_name) = &opts.name {
        tar_filename.push_str(custom_name);
    } else {
        tar_filename.push_str(&generate_randname_with_time()?);
    }
    let tarfile = compress_tar_gz_target(&temp_dir, &dstpath.path, tar_filename)?;
    // remove temp directory
    std::fs::remove_dir_all(&temp_dir).map_err(|e| anyhow!("Remove ERROR: {e}"))?;

    println!("Backup complete: {tarfile:?}");

    Ok(())
}

fn process_resotres(bkfile: PathBuf) -> Result<()> {
    // print file create time info
    println!("Backup file created at: {}", get_file_createtime(&bkfile)?);

    let decompress_path = decompress_tar_gz_target(&bkfile)?;
    debug!("decompress_path: {:?}", decompress_path);

    let config_path = decompress_path.join("config.yaml");
    let tasks = parse_config(config_path.to_str().unwrap())?;

    // show tasks infos
    tasks.dump_tasks(&decompress_path);

    // confirm restore
    let mut input = String::new();
    print!("Do you want to restore? [Y/n]");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_ascii_lowercase() != "y" && input != "\n" {
        println!("Canceled.");
        return Ok(());
    }

    let backup_dir = generate_tempdir(&PathBuf::from("/tmp"))?;
    for task in tasks {
        task.restore(&backup_dir, &decompress_path)?;
    }
    // copy config file to temp directory
    let config_path = decompress_path.join("config.yaml");
    std::fs::copy(
        &config_path,
        backup_dir.join(config_path.file_name().unwrap()),
    )
    .map_err(|e| anyhow!("Copy ERROR: {e}"))?;

    // compress old files
    let tar_filename = format!("restore_bk_{}", generate_randname_with_time()?);
    let tarfile = compress_tar_gz_target(&backup_dir, &PathBuf::from("./bkup"), tar_filename)?;

    // remove temp directory
    std::fs::remove_dir_all(&backup_dir).map_err(|e| anyhow!("Remove ERROR: {e}"))?;
    std::fs::remove_dir_all(&decompress_path).map_err(|e| anyhow!("Remove ERROR: {e}"))?;

    println!("Restore complete. Old files are compressed: {tarfile:?}");
    Ok(())
}

pub fn run() -> Result<()> {
    let opts = Opts::parse();

    match opts.mode {
        Mode::Backup => {
            info!("Backup mode");
            let tasks = parse_config(&opts.config)?;
            process_backups(tasks, &opts)?;
        }
        Mode::Restore => {
            info!("Restore mode");
            if let Some(bkfile) = opts.bkfile {
                let bkfile = MyPathBuf::from_str(&bkfile)?;
                process_resotres(bkfile.path)?;
            } else {
                eprintln!("bkfile is required");
            }
        }
    }

    Ok(())
}
