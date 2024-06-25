use std::fs::File;
use std::io::{BufRead, Error, Read as _};
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use anyhow::Result;
use goblin::mach::{Mach, MachO};
use goblin::Object;
#[cfg(target_os="linux")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os="linux")]
use tar::{EntryType,Archive};
#[cfg(target_os="linux")]
use flate2::read::GzDecoder;

#[cfg(target_os="macos")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_os="macos")]
use tar::{EntryType,Archive};
#[cfg(target_os="macos")]
use flate2::read::GzDecoder;

#[cfg(target_os="windows")]
use zip::ZipArchive;
#[cfg(target_os="windows")]
use zip;

/// tag 是否解压到当前目录，
/// 需要解压的文件位置test.zip
/// 如果当前目录是a，则将test中的内容全部解压的a中，避免文件移动出现权限问题
#[cfg(target_os = "windows")]
pub fn unzip(zip_path: &str, dest: &str, tag: bool) -> Result<(), Error> {
    let file: File = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let filename = file.name().replace("\\\\", "/");
        let index = filename.find("/").unwrap();
        let filepath = if tag {
            format!("{}/{}", dest, filename[index + 1..].to_string())
        } else {
            format!("{}/{}", dest, filename)
        };

        if file.is_dir() {
            fs::create_dir_all(&filepath)?
        } else {
            if Path::new(&filepath).parent().is_none() {
                fs::create_dir_all(&filepath)?;
            }

            let mut extracted_file = File::create(&filepath)?;
            io::copy(&mut file, &mut extracted_file)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn untar(tar_gz_path: &str, dest: &str, tag: bool) -> Result<(), Error> {
    let tar_gz = File::open(tar_gz_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let filename = path.to_string_lossy();

        let filepath = if tag {
            format!(
                "{}/{}",
                dest,
                filename.split('/').skip(1).collect::<Vec<_>>().join("/")
            )
        } else {
            format!("{}/{}", dest, filename)
        };

        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(&filepath)?;
            }
            EntryType::Regular => {
                if let Some(parent) = Path::new(&filepath).parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                let mut extracted_file = File::create(&filepath)?;
                io::copy(&mut entry, &mut extracted_file)?;

                // Set file permissions
                let mode = entry.header().mode()?;
                fs::set_permissions(&filepath, fs::Permissions::from_mode(mode))?;
            }
            EntryType::Symlink => {
                let target = entry.link_name()?;
                if let Some(target) = target {
                    std::os::unix::fs::symlink(target, &filepath)?;
                }
            }
            EntryType::Link => {
                let target = entry.link_name()?;
                if let Some(target) = target {
                    fs::hard_link(target, &filepath)?;
                }
            }
            _ => {
                // Handle other entry types if necessary
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn untar(tar_gz_path: &str, dest: &str, tag: bool) -> Result<(), Error> {
    let tar_gz = File::open(tar_gz_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let filename = path.to_string_lossy();

        let filepath = if tag {
            format!(
                "{}/{}",
                dest,
                filename.split('/').skip(1).collect::<Vec<_>>().join("/")
            )
        } else {
            format!("{}/{}", dest, filename)
        };

        match entry.header().entry_type() {
            EntryType::Directory => {
                fs::create_dir_all(&filepath)?;
            }
            EntryType::Regular => {
                if let Some(parent) = Path::new(&filepath).parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                let mut extracted_file = File::create(&filepath)?;
                io::copy(&mut entry, &mut extracted_file)?;

                // Set file permissions
                let mode = entry.header().mode()?;
                fs::set_permissions(&filepath, fs::Permissions::from_mode(mode))?;
            }
            EntryType::Symlink => {
                let target = entry.link_name()?;
                if let Some(target) = target {
                    std::os::unix::fs::symlink(target, &filepath)?;
                }
            }
            EntryType::Link => {
                let target = entry.link_name()?;
                if let Some(target) = target {
                    fs::hard_link(target, &filepath)?;
                }
            }
            _ => {
                // Handle other entry types if necessary
            }
        }
    }

    Ok(())
}

pub fn read_lines(path: &str) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);

    let mut res: Vec<String> = Vec::new();
    loop {
        let mut tmp = String::new();
        let size = reader.read_line(&mut tmp)?;
        if size <= 0 {
            break;
        }
        res.push(tmp);
    }
    Ok(res)
}

pub fn exists(path: &str) -> bool {
    let filepath = Path::new(path);
    match fs::metadata(filepath) {
        Ok(_) => true,
        Err(_) => {
            return false;
        }
    }
}

#[cfg(target_os="linux")]
pub fn get_executable_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 获取当前可执行文件的路径
    let exe_path: PathBuf = env::current_exe()?;
    
    // 检查是否是软链接
    match fs::read_link(&exe_path) {
        Ok(link_path) => {
            // 如果是软链接，返回软链接的源地址
            Ok(link_path)
        }
        Err(_) => {
            // 如果不是软链接，返回可执行文件本身的地址
            Ok(exe_path)
        }
    }
}

#[cfg(target_os="macos")]
pub fn get_executable_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 获取当前可执行文件的路径
    let exe_path: PathBuf = env::current_exe()?;
    
    // 检查是否是软链接
    match fs::read_link(&exe_path) {
        Ok(link_path) => {
            // 如果是软链接，返回软链接的源地址
            Ok(link_path)
        }
        Err(_) => {
            // 如果不是软链接，返回可执行文件本身的地址
            Ok(exe_path)
        }
    }
}

#[cfg(test)]
#[test]
#[cfg(target_os="windows")]
fn test_un_zip() {
    let src = r"E:\Code\tools\nvm\temp\npm-v9.8.1.zip".to_string();
    let dest = r"E:\Code\tools\nvm\temp\nvm-npm".to_string();
    let res = unzip(&src, &dest, false);
    if res.is_err() {
        println!("{:?}", res.err().unwrap())
    }
}

#[cfg(test)]
#[test]
#[cfg(target_os="windows")]
fn test_file_exists() {
    let path = r"D:\workspace\rust\nvm-win-rust";
    assert_eq!(exists(path), true);
}

#[cfg(test)]
#[test]
#[cfg(target_os="linux")]
fn test_get_executable_path(){
    let path = get_executable_path();
    if path.is_err(){
        println!("{:?}",path.err());
    }else{
        let mut path = path.unwrap();
        println!("exe path: {}",path.to_str().unwrap());
        path.pop();
        println!("exe dir: {}",path.to_str().unwrap());
    }
}