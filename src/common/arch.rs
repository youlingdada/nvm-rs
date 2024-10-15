use goblin::mach::Mach;
use goblin::Object;

use std::env;
use std::fs::File;
use std::io::Read;

// get the bit of the file
pub fn bit(path: &str) -> &str {
    let res = File::open(path);
    if res.is_err() {
        println!("Open file fail, err:{:?}", res.as_ref().err());
        return "?";
    }
    let mut file = res.unwrap();
    let mut buffer = Vec::new();
    if let Err(err) = file.read_to_end(&mut buffer) {
        println!("Read File content fail, err:{:?}", err);
        return "?";
    }

    let res = Object::parse(&buffer);
    if res.is_err() {
        println!("File header parse fail, err:{:?}", res.as_ref().err());
        return "?";
    }

    let obj = res.unwrap();
    match obj {
        Object::Elf(elf) => {
            return if elf.is_64 { "64" } else { "32" };
        }
        Object::PE(pe) => {
            return if pe.is_64 { "64" } else { "32" };
        }
        Object::Mach(mach) => match mach {
            Mach::Binary(b) => {
                return if b.is_64 { "64" } else { "32" };
            }
            _ => {
                println!("Not support file format");
                return "?";
            }
        },
        _ => {
            println!("Unknown file format");
            return "?";
        }
    }
}

// get the current arch
pub fn arch_map() -> String {
    let a = env::consts::ARCH;

    let node_arch = match a {
        "x86" => "x86",
        "x86_64" => "x64",
        "arm" => "armv6",
        "armv7" => "armv7",
        "aarch64" => "arm64",
        _ => {
            println!("Not support this arch: {}", a);
            "?"
        }
    };
    return node_arch.to_string();
}

// validate the input arch, if it is empty, return the current arch
pub fn validate(str: &str) -> String {
    let mut tmp = str.to_string();
    if str.is_empty() {
        tmp = std::env::consts::ARCH.to_string();
    }
    if tmp.contains("64") {
        "64".to_string()
    } else {
        "32".to_string()
    }
}

#[cfg(test)]
#[test]
#[cfg(target_os = "windows")]
fn test_validate() {
    let res = validate("");
    assert_eq!(res, "64");
}

#[cfg(test)]
#[test]
#[cfg(target_os = "windows")]
fn test_bit() {
    let path = "assets/64bit.exe";
    let res = bit(path);
    assert_eq!(res, "64");
}
