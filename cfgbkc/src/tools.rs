use std::fs::File;
use flate2::write::GzEncoder;
use flate2::Compression;
use anyhow::Result;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn compress_tar_gz_target(target_path: &PathBuf, save_path: &PathBuf) -> Result<()> {
    let tar_filename = format!("bkup_{}.tar.gz", generate_timestamp()?);
    let tar_file = target_path.join(tar_filename);
    let tar_gz = File::create(tar_file)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", target_path)?;
    Ok(())
}

pub fn generate_tempdir(path: &PathBuf) -> Result<PathBuf> {
    let tempdir_name = format!("tmp_{}", generate_timestamp()?);
    let tempdir = path.join(tempdir_name);
    std::fs::create_dir_all(&tempdir)?;
    Ok(tempdir)
}

pub fn generate_timestamp() -> Result<String> {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
    Ok(format!("{}", since_the_epoch.as_secs()))
}