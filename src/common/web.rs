use crate::common::{arch, file};
use crate::Environment;
use reqwest::{Client, ClientBuilder, Proxy, StatusCode};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::{cmp::Ordering, path::MAIN_SEPARATOR};
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct WebContext {
    node_base_address: String,
    npm_base_address: String,
    rt: Runtime,
    client: Client,
}

const NODE_BASE_ADDRESS: &str = "https://nodejs.org/dist/";
const NPM_BASE_ADDRESS: &str = "https://github.com/npm/cli/archive/";

impl WebContext {
    pub fn new() -> WebContext {
        let rt = Runtime::new();
        if rt.is_err() {
            println!(
                "create async runtime println. {}",
                rt.as_ref().err().unwrap()
            )
        }

        WebContext {
            node_base_address: NODE_BASE_ADDRESS.to_owned(),
            npm_base_address: NPM_BASE_ADDRESS.to_owned(),
            rt: rt.unwrap(),
            client: Client::new(),
        }
    }

    pub fn set_mirrors(&mut self, node_mirror: &str, npm_mirror: &str) {
        let mut node_base_address = String::new();
        if node_mirror != "" && node_mirror != "none" {
            node_base_address = node_mirror.to_string();

            if node_base_address.to_lowercase().starts_with("http") {
                node_base_address = "http://".to_string() + node_base_address.as_ref();
            }
            if !node_base_address.ends_with("/") {
                node_base_address = node_base_address + "/";
            }
            self.node_base_address = node_base_address;
        }
        let mut npm_base_address: String = String::new();
        if npm_mirror != "" && npm_mirror != "none" {
            npm_base_address = npm_mirror.to_string();

            if npm_base_address.to_lowercase().starts_with("http") {
                npm_base_address = "http://".to_string() + npm_base_address.as_ref();
            }
            if !npm_base_address.ends_with("/") {
                npm_base_address = npm_base_address + "/";
            }
            self.npm_base_address = npm_base_address;
        }
    }

    pub fn get_full_node_url(&self, path: &str) -> String {
        self.node_base_address.clone() + path
    }

    pub fn get_full_npm_url(&self, path: &str) -> String {
        self.npm_base_address.clone() + path
    }

    pub fn set_proxy(&mut self, p: &str, verify_ssl: bool) -> Result<(), Box<dyn Error>> {
        let mut builder = ClientBuilder::new().danger_accept_invalid_certs(!verify_ssl);

        if !p.is_empty() && p != "none" {
            let url = Url::parse(p)?;
            let proxy = if p.starts_with("https") {
                Proxy::https(url)?
            } else {
                Proxy::http(url)?
            };
            builder = builder.proxy(proxy);
        }
        self.client = builder.build()?;
        Ok(())
    }

    fn ping(&self, url: &String) -> bool {
        let req_builder = self.client.head(url).header("User-Agent", "NVM WIN RUST");

        let resp = self.rt.block_on(req_builder.send());

        if resp.is_err() {
            println!("ping err:{}", resp.err().unwrap());
            return false;
        }
        let response = resp.unwrap();
        response.status().eq(&StatusCode::OK)
    }

    pub fn download(&self, url: &str, target: &str, version: &str) -> bool {
        let result = File::create(target);
        if result.is_err() {
            println!(
                "errror: while creating:{}-{}",
                target,
                result.err().unwrap()
            );
            return false;
        }

        let builder = self.client.get(url).header("User-Agent", "NVM WIN RUST");
        let resp = self.rt.block_on(builder.send());

        if resp.is_err() {
            println!(
                "errror: while downloading {} - {}",
                url,
                resp.err().unwrap()
            );
            return false;
        }

        let resp = resp.unwrap();
        let headers = resp.headers().clone();
        let status = resp.status().clone();
        let response = self.rt.block_on(resp.bytes());

        if response.is_err() {
            println!("Http body read failed. {}", response.err().unwrap());
            return false;
        }

        let mut file = result.unwrap();
        let body = response.unwrap();

        if let Some(err) = file.write_all(&body).err() {
            println!("Failed to read response body: {}", err.to_string());
            return false;
        }

        let redirect = headers.get("Location");
        if redirect.is_some() {
            let redirect_url = redirect.unwrap().to_str().unwrap().to_string();
            match status {
                StatusCode::MULTIPLE_CHOICES => {
                    if redirect_url.len() > 0 && !redirect_url.eq(url) {
                        let res = self.download(&redirect_url, target, version);
                        return res;
                    }

                    if redirect_url.contains("/npm/cli/archive/v6.14.17.zip") {
                        let url =
                            "https://github.com/npm/cli/archive/refs/tags/v6.14.17.zip".to_string();
                        let res = self.download(&url, target, version);
                        return res;
                    }
                    println!(
                        "\n\nRemote server failure\n\n---\nGet {} ---> {}\n\n",
                        url, status
                    );

                    for (k, v) in headers.iter() {
                        println!("{}: {}\n", k.to_string(), v.to_str().unwrap().to_string())
                    }
                    if body.len() > 0 {
                        let str = String::from_utf8(body.to_vec()).unwrap();
                        println!("\n{}", str)
                    }
                    println!("\n---\n\n");
                    return false;
                }
                StatusCode::FOUND => {}
                StatusCode::TEMPORARY_REDIRECT => {
                    println!("Redirecting to {}", redirect_url);
                    let res = self.download(&redirect_url, target, version);
                    return res;
                }
                StatusCode::OK => {}
                _ => {
                    println!("Download failed. Rolling Back.");
                    if let Err(err) = fs::remove_dir(target) {
                        println!("{}", target);
                        println!("Rollback failed. {}", err)
                    }
                    return false;
                }
            }
        }
        true
    }

    pub fn get_remote_text_file(&self, url: &str) -> String {
        let result = self.rt.block_on(self.client.get(url).send());
        if result.is_err() {
            println!("\nCould not retrieve {}.\n\n", url);
            println!("{}", result.err().unwrap());
            exit(1);
        }

        let body = result.ok().unwrap();
        let res = self.rt.block_on(body.bytes());
        if res.is_err() {
            println!("{}", res.err().unwrap());
            exit(1);
        }
        let contents = String::from_utf8(res.unwrap().to_vec());
        if contents.is_err() {
            println!("{}", contents.err().unwrap());
            exit(1);
        }
        contents.unwrap()
    }

    pub fn get_node_pre(v: &str) -> String {
        #[cfg(target_os = "windows")]
        let os_name = "win";
        #[cfg(target_os = "linux")]
        let os_name = "linux";
        #[cfg(target_os = "macos")]
        let os_name = "darwin";

        let mut main = 0;
        if let Some(main_str) = v.split(".").next() {
            main = main_str.parse().unwrap();
        }

        let node_arch = arch::arch_map();

        if main > 0 {
            format!("{}-{}", os_name, node_arch)
        } else {
            "".to_string()
        }
    }

    pub fn get_node_js(&self, root: &str, v: &str, a: &str, append: bool) -> bool {
        let v_pre = Self::get_node_pre(v);

        let url = self.get_node_url(v, &v_pre, &a, append);
        if url.eq("") {
            println!("Node.js v{} {} bit isn't available right now.", v, a);
        } else {
            #[cfg(target_os = "windows")]
            let mut file_name = format!("{}\\v{}\\node{}.exe", root, v, a);
            #[cfg(target_os = "windows")]
            if url.ends_with(".zip") {
                file_name = format!("{}\\v{}\\node.zip", root, v);
            }

            #[cfg(target_os = "linux")]
            let mut file_name = format!("{}/v{}/node{}", root, v, a);
            #[cfg(target_os = "linux")]
            if url.ends_with(".tar.gz") {
                file_name = format!("{}/v{}/node.tar.gz", root, v);
            }

            #[cfg(target_os = "macos")]
            let mut file_name = format!("{}/v{}/node{}", root, v, a);
            #[cfg(target_os = "macos")]
            if url.ends_with(".tar.gz") {
                file_name = format!("{}/v{}/node.tar.gz", root, v);
            }

            println!("Downloading node.js version {} ({}-bit)..", v, a);

            return if self.download(&url, &file_name, v) {
                #[cfg(target_os = "windows")]
                let root_v = format!("{}\\v{}", root, v);
                #[cfg(target_os = "linux")]
                let root_v = format!("{}/v{}", root, v);
                #[cfg(target_os = "macos")]
                let root_v = format!("{}/v{}", root, v);

                // Extract the zip file
                if url.ends_with("zip") || url.ends_with("tar.gz") {
                    println!("Extracting node and npm..");

                    #[cfg(target_os = "windows")]
                    let res = file::unzip(&file_name, &root_v, true);

                    #[cfg(target_os = "linux")]
                    let res = file::untar(&file_name, &root_v, true);

                    #[cfg(target_os = "macos")]
                    let res = file::untar(&file_name, &root_v, true);

                    if fs::remove_file(&file_name).is_err() {
                        println!("Failed to remove {} after successful extraction. Please remove manually.",file_name);
                    }

                    if res.is_err() {
                        println!("unzip or untar fail, err:{:?}", res.err());
                        return false;
                    }
                }
                println!("Complete");
                true
            } else {
                false
            };
        }
        true
    }

    pub fn get_npm(&self, root: &str, v: &str) -> bool {
        #[cfg(target_os = "windows")]
        let path = format!("v{}.zip", v);

        #[cfg(target_os = "linux")]
        let path = format!("v{}.tar.gz", v);

        #[cfg(target_os = "macos")]
        let path = format!("v{}.tar.gz", v);

        let url = self.get_full_npm_url(&path);

        let temp_dir = format!("{}{}temp", root, MAIN_SEPARATOR);
        if !file::exists(&temp_dir) {
            println!("Creating {}\n", temp_dir);
            let result = fs::create_dir(temp_dir.clone());
            if result.is_err() {
                println!("create npm temp dir failed.{}", result.err().unwrap());
                exit(1);
            }
        }
        #[cfg(target_os = "windows")]
        let file_name = format!("{}{}npm-v{}.zip", temp_dir, MAIN_SEPARATOR, v);

        #[cfg(target_os = "linux")]
        let file_name = format!("{}{}npm-v{}.tar.gz", temp_dir, MAIN_SEPARATOR, v);

        #[cfg(target_os = "macos")]
        let file_name = format!("{}{}npm-v{}.tar.gz", temp_dir, MAIN_SEPARATOR, v);

        println!("Downloading npm version {}...", v);
        if self.download(&url, &file_name, v) {
            println!("Complete\n");
            return true;
        } else {
            false
        }
    }

    pub fn get_node_url(&self, v: &str, v_pre: &str, arch: &str, append: bool) -> String {
        let mut url = String::new();
        if !append {
            let version = semver::Version::parse(&v);
            if version.is_err() {
                println!(
                    "Node.js v{} {} bit isn't available right now.{}",
                    v,
                    arch,
                    version.err().unwrap()
                );
                exit(1);
            }
            let version = version.unwrap();

            let core_pack = semver::Version::new(16, 9, 0);

            #[cfg(target_os = "windows")]
            let path = format!("v{}/node-v{}-{}.zip", v, v, v_pre);
            #[cfg(target_os = "linux")]
            let path = format!("v{}/node-v{}-{}.tar.gz", v, v, v_pre);

            #[cfg(target_os = "macos")]
            let path = format!("v{}/node-v{}-{}.tar.gz", v, v, v_pre);

            match version.cmp_precedence(&core_pack) {
                Ordering::Equal => {
                    url = self.get_full_node_url(&path);
                }
                Ordering::Greater => {
                    url = self.get_full_node_url(&path);
                }
                _ => {}
            }
        }

        #[cfg(target_os = "windows")]
        if url == "" {
            url = self.get_full_node_url(&format!("v{}/{}/node.exe", v, v_pre));
        }

        #[cfg(target_os = "windows")]
        if let Err(err) = self.rt.block_on(self.client.head(&url).send()) {
            println!("a 64 bit {} not exists,err:{}", v, err);
            return "".to_string();
        }
        url
    }

    pub fn is_node64_bid_available(v: &str) -> bool {
        if v == "latest" {
            return true;
        }
        let binding = v.to_string();
        let arr: Vec<&str> = binding.split(".").collect();
        let (mut main, mut minor) = (0, 0);
        if let Ok(val) = arr[0].to_string().parse() {
            main = val;
        }
        if let Ok(val) = arr[1].to_string().parse() {
            minor = val;
        }
        if main == 0 && minor < 8 {
            return false;
        }
        true
    }
}

impl Default for WebContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[test]
fn test_set_mirrors() {
    let mut address = WebContext::new();
    address.set_mirrors(&"a.com".to_string(), &"b.com".to_string());
    println!("{:?}", address)
}

#[cfg(test)]
#[test]
fn test_ping() {
    let http_context = WebContext::new();
    let url = "https://github.com/npm/cli/archive/refs/tags/v6.14.17.zip".to_string();
    let x = http_context.ping(&url);
    assert_eq!(x, true);
}

#[cfg(test)]
#[test]
fn test_download_url() {
    let url = "https://github.com/npm/cli/archive/refs/tags/v6.14.17.zip".to_string();
    let http_context = WebContext::new();
    let (target, version) = (
        r"D:\workspace\Rust\nvm-win-rust\res.zip".to_string(),
        "16.14.0".to_string(),
    );

    let x = http_context.download(&url, &target, &version);
    println!("{}", x)
}
