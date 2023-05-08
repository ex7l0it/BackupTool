use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rand::Rng;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

pub fn compress_tar_gz_target(
    target_path: &Path,
    save_path: &Path,
    filename: String,
) -> Result<PathBuf> {
    if !save_path.exists() {
        std::fs::create_dir_all(save_path)?;
    }
    let tar_filename = format!("{filename}.tar.gz");
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
    let tempdir_name = format!("tmp_{}", generate_random_hexstring(16)?);
    let tempdir = path.join(tempdir_name);
    std::fs::create_dir_all(&tempdir)?;
    Ok(tempdir)
}

pub fn generate_random_hexstring(len: usize) -> Result<String> {
    let mut randstr = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..len {
        let num: u8 = rng.gen();
        randstr.push_str(&format!("{num:02x}"));
    }
    Ok(randstr)
}

pub fn generate_randname_with_time() -> Result<String> {
    let now_time = Local::now();
    let randname = format!(
        "{}_{}",
        now_time.format("%Y%m%d_%H%M%S"),
        generate_random_hexstring(8)?
    );
    Ok(randname)
}

pub fn get_file_createtime(file: &Path) -> Result<String> {
    let file = File::open(file).map_err(|e| anyhow!("Open file ERROR: {file:?} - {e}"))?;
    let create_time = file.metadata()?.created()?;
    let datetime: DateTime<Local> = create_time.into();
    Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
}
