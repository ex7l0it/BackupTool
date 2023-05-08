use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Archive;

pub fn compress_tar_gz_target(target_path: &PathBuf, save_path: &Path) -> Result<PathBuf> {
    if !save_path.exists() {
        std::fs::create_dir_all(save_path)?;
    }
    let tar_filename = format!("bkup_{}.tar.gz", generate_timestamp()?);
    let tar_file = save_path.join(tar_filename);
    let tar_gz = File::create(&tar_file)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(".", target_path)?;
    Ok(tar_file)
}

pub fn decompress_tar_gz_target(tar_file: &Path) -> Result<PathBuf> {
    let tar_gz =
        File::open(tar_file).map_err(|e| anyhow!("Open tar file Failed: {tar_file:?} - {e}"))?;
    let tempdir = generate_tempdir(&PathBuf::from("/tmp"))?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive
        .unpack(&tempdir)
        .map_err(|e| anyhow!("Unpack tar file Failed: {tar_file:?} - {e}"))?;
    Ok(tempdir)
}

pub fn generate_tempdir(path: &Path) -> Result<PathBuf> {
    let tempdir_name = format!("tmp_{}", generate_timestamp()?);
    let tempdir = path.join(tempdir_name);
    std::fs::create_dir_all(&tempdir)?;
    Ok(tempdir)
}

pub fn generate_timestamp() -> Result<String> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Ok(format!("{}", since_the_epoch.as_secs()))
}
