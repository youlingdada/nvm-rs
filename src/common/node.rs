use crate::common::web::WebContext;
use crate::common::{arch, file};
use chrono::NaiveDate;
use regex::Regex;
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug)]
struct NodeInfo {
    version: Version,
    date: NaiveDate,
    files: Vec<String>,
    npm: String,
    v8: String,
    uv: String,
    z_lib: String,
    open_ssl: String,
    modules: u32,
    lts: bool,
    security: bool,
}

impl NodeInfo {
    fn is_lts(&self) -> bool {
        self.lts
    }

    fn is_current(&self) -> bool {
        if self.is_lts() {
            return false;
        }

        let benchmark = semver::Version::parse("1.0.0").unwrap();
        if self.version < benchmark {
            return false;
        }
        return true;
    }

    fn is_stable(&self) -> bool {
        if self.is_current() {
            return false;
        }
        if self.version.major != 0 {
            return false;
        }
        return self.version.minor % 2 == 0;
    }

    fn is_unstable(&self) -> bool {
        if self.is_stable() {
            return false;
        }
        if self.version.major != 0 {
            return false;
        }
        return self.version.minor % 2 != 0;
    }

    fn parse_node_info(json_value: Value) -> NodeInfo {
        let version: String = json_value["version"]
            .to_string()
            .replace("v", "")
            .replace("\"", "");
        let version = Version::parse(version.as_str()).unwrap();

        let date = json_value["date"].as_str().unwrap();
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();

        let files = json_value["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let npm = json_value["npm"].to_string().replace("\"", "");

        let v8 = json_value["v8"].to_string();

        let uv = json_value["uv"].to_string();

        let z_lib = json_value["zlib"].to_string();

        let open_ssl = json_value["openssl"].to_string();

        let modules = json_value["modules"].as_u64().unwrap_or(0) as u32;

        let lts = json_value["lts"].as_bool().unwrap_or(true);

        let security = json_value["security"].as_bool().unwrap();

        if version.to_string() == "0.11.16" {
            println!();
        }

        NodeInfo {
            version,
            date,
            files,
            npm,
            v8,
            uv,
            z_lib,
            open_ssl,
            modules,
            lts,
            security,
        }
    }
}

pub fn get_current_version() -> (String, String) {
    let result = Command::new("node").arg("-v").output();
    if result.is_err() {
        println!("node not exists, err:{}", result.err().unwrap());
        return ("Unknown".to_string(), "".to_string());
    }
    let res = String::from_utf8(result.unwrap().stdout);
    if res.is_err() {
        println!("{}", res.err().unwrap());
        return ("Unknown".to_string(), "".to_string());
    }
    let str = res.ok();
    if str.is_none() {
        return ("Unknown".to_string(), "".to_string());
    }
    let mut str = str
        .unwrap()
        .trim_end_matches("\r\n")
        .trim_start_matches("\r\n")
        .to_string();
    let mut regx = regex::Regex::new("v").unwrap();
    str = regx.replace_all(str.as_str(), "").to_string();
    regx = regex::Regex::new("-.*$").unwrap();
    str = regx.replace_all(str.as_str(), "").to_string();
    let v = str
        .trim_start_matches("\r\n")
        .trim_end_matches("\r\n")
        .to_string();

    let out = Command::new("node")
        .arg("-p")
        .arg("console.log(process.execPath)")
        .output()
        .ok()
        .unwrap();
    let mut str_p = String::from_utf8(out.stdout).ok().unwrap();
    str_p = regex::Regex::new("undefined")
        .unwrap()
        .replace_all(str_p.as_str(), "")
        .to_string();
    let file = str_p
        .trim_start_matches("\n")
        .trim_end_matches("\n")
        .to_string();

    let mut bit = arch::bit(&file);
    if bit == "?" {
        let out = Command::new("node")
            .arg("-v")
            .arg("console.log(process.arch)")
            .output()
            .ok()
            .unwrap();
        let res = String::from_utf8(out.stdout);
        if res.is_err() {
            println!("{}", res.err().unwrap());
            return ("Unknown".to_string(), "".to_string());
        }
        let str = res.ok().unwrap();
        if str.as_str() == "x64" {
            bit = "64"
        } else {
            bit = "32"
        }
    }
    (v, bit.to_string())
}

#[cfg(target_os = "windows")]
pub fn is_version_installed(root: &str, version: &str, cpu: &str) -> bool {
    let e32 = file::exists(&format!("{}\\v{}\\node32.exe", root, version));
    let e64 = file::exists(&format!("{}\\v{}\\node64.exe", root, version));
    let used = file::exists(&format!("{}\\v{}\\node.exe", root, version));
    if cpu == "all" {
        return ((e32 || e64) && used) || e32 && e64;
    }
    let mut node_path = format!("{}\\v{}\\node{}.exe", root, version, cpu);
    if file::exists(&node_path) {
        return true;
    }
    if ((e32 || e64) && used) || (e32 && e64) {
        return true;
    }
    node_path = format!("{}\\v{}\\node.exe", root, version);
    if !e32 && !e64 && used && arch::validate(cpu) == arch::bit(&node_path) {
        return true;
    }
    if cpu == "32" {
        return e32;
    }
    if cpu == "64" {
        return e64;
    }
    false
}

#[cfg(target_os = "linux")]
pub fn is_version_installed(root: &str, version: &str, cpu: &str) -> bool {
    let e32 = file::exists(&format!("{}/v{}/bin/node32", root, version));
    let e64 = file::exists(&format!("{}/v{}/bin/node64", root, version));
    let used = file::exists(&format!("{}/v{}/bin/node", root, version));
    if cpu == "all" {
        return ((e32 || e64) && used) || e32 && e64;
    }
    let mut node_path = format!("{}/v{}/bin/node{}", root, version, cpu);
    if file::exists(&node_path) {
        return true;
    }
    if ((e32 || e64) && used) || (e32 && e64) {
        return true;
    }
    node_path = format!("{}/v{}/bin/node", root, version);
    if !e32 && !e64 && used && arch::validate(cpu) == arch::bit(&node_path) {
        return true;
    }
    if cpu == "32" {
        return e32;
    }
    if cpu == "64" {
        return e64;
    }
    false
}

#[cfg(target_os = "macos")]
pub fn is_version_installed(root: &str, version: &str, cpu: &str) -> bool {
    let e32 = file::exists(&format!("{}/v{}/bin/node32", root, version));
    let e64 = file::exists(&format!("{}/v{}/bin/node64", root, version));
    let used = file::exists(&format!("{}/v{}/bin/node", root, version));
    if cpu == "all" {
        return ((e32 || e64) && used) || e32 && e64;
    }
    let mut node_path = format!("{}/v{}/bin/node{}", root, version, cpu);
    if file::exists(&node_path) {
        return true;
    }
    if ((e32 || e64) && used) || (e32 && e64) {
        return true;
    }
    node_path = format!("{}/v{}/bin/node", root, version);
    if !e32 && !e64 && used && arch::validate(cpu) == arch::bit(&node_path) {
        return true;
    }
    if cpu == "32" {
        return e32;
    }
    if cpu == "64" {
        return e64;
    }
    false
}

pub fn is_version_available(v: &str, web_ctx: &WebContext) -> bool {
    let tmp = Version::parse(v).unwrap();
    let (avail, _, _, _, _, _) = get_available(web_ctx);
    for b in avail {
        if b.eq(&tmp) {
            return true;
        }
    }
    false
}

pub fn get_installed(root: &str) -> Vec<String> {
    let mut list: Vec<Version> = Vec::new();
    let result = fs::read_dir(root).unwrap();
    for f in result {
        let d = f.unwrap();
        let mut file_name = d.file_name().into_string().unwrap();
        let info = d.metadata().unwrap();
        if info.is_dir() || (info.is_symlink()) {
            let re = regex::Regex::new("v").unwrap();
            if re.is_match(file_name.as_str()) {
                file_name = file_name.replace("v", "");
                let current_version = semver::Version::parse(file_name.as_str()).unwrap();
                list.push(current_version);
            }
        }
    }
    list.sort();

    let mut log_gable_list: Vec<String> = Vec::new();
    for v in list {
        log_gable_list.push(format!("v{}", v.to_string()));
    }
    log_gable_list.reverse();
    log_gable_list
}

pub fn get_available(
    web_context: &WebContext,
) -> (
    Vec<Version>,
    Vec<Version>,
    Vec<Version>,
    Vec<Version>,
    Vec<Version>,
    HashMap<String, String>,
) {
    let mut all: Vec<Version> = Vec::new();
    let mut lts: Vec<Version> = Vec::new();
    let mut current: Vec<Version> = Vec::new();
    let mut stable: Vec<Version> = Vec::new();
    let mut unstable: Vec<Version> = Vec::new();

    let mut npm: HashMap<String, String> = HashMap::new();
    let url = web_context.get_full_node_url("index.json");
    let text = web_context.get_remote_text_file(url.as_str());

    // 反序列化 JSON 字符串为 NodeInfo 结构体的 Vec
    let list: Vec<Value> = serde_json::from_str(&text).unwrap();
    for v in list {
        let node = NodeInfo::parse_node_info(v);

        all.push(node.version.clone());
        npm.insert(node.version.to_string(), node.npm.clone());

        if node.is_lts() {
            lts.push(node.version.clone())
        } else if node.is_current() {
            current.push(node.version.clone())
        } else if node.is_stable() {
            stable.push(node.version.clone())
        } else if node.is_unstable() {
            unstable.push(node.version.clone())
        }
    }
    (all, lts, current, stable, unstable, npm)
}

pub fn get_npm_version(node_version: &str, web_context: &WebContext) -> String {
    let (_, _, _, _, _, npm) = get_available(web_context);
    npm.get(&node_version.to_string()).unwrap().to_string()
}

pub fn get_latest(web_context: &WebContext) -> Option<String> {
    let url = web_context.get_full_node_url("latest/SHASUMS256.txt");
    let content = web_context.get_remote_text_file(url.as_str());

    let re = Regex::new("node-v(.+)+msi").unwrap();
    let reg = Regex::new("node-v|-[xa].+").unwrap();

    if let Some(res) = re.find(content.as_str()) {
        Some(reg.replace_all(res.as_str(), "").to_string())
    } else {
        None
    }
}

pub fn get_lts(web_context: &WebContext) -> Option<String> {
    let (_, lts_list, _, _, _, _) = get_available(web_context);
    if let Some(v) = lts_list.get(0) {
        return Some(v.to_string());
    }
    None
}

#[cfg(test)]
#[test]
fn test_get_current_version() {
    let _  = get_current_version();
}

#[cfg(test)]
#[test]
fn test_get_available() {
    let web_ctx = WebContext::new();
    let _ = get_available(&web_ctx);
}

#[cfg(test)]
#[test]
fn test_is_version_available() {
    let web_ctx: WebContext = WebContext::new();
    assert_eq!(is_version_available("14.16.0", &web_ctx), true);
}
