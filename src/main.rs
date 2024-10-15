use std::collections::HashMap;

use std::{env, fs};

use std::path::{PathBuf, MAIN_SEPARATOR};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use common::node::{get_latest, get_lts};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use regex::Regex;
use semver::Version;

use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::Table;
#[cfg(target_os = "windows")]
use windows::core::Result as WinResult;
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, MessageBoxW, MB_OK};

#[cfg(target_os = "windows")]
use winapi::um::consoleapi::GetConsoleMode;
#[cfg(target_os = "windows")]
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(target_os = "windows")]
use winapi::um::processenv::GetStdHandle;
#[cfg(target_os = "windows")]
use winapi::um::winbase::STD_OUTPUT_HANDLE;

use common::{arch, cmd, strings};

use crate::common::file;
use crate::common::filepath;
use crate::common::node;
use crate::common::web::WebContext;

mod common;

#[derive(Debug)]
struct Context {
    web_ctx: WebContext,
}

impl Context {
    fn new() -> Context {
        Context {
            web_ctx: WebContext::new(),
        }
    }
}

#[derive(Debug)]
struct Environment {
    settings: String,
    root: String,
    symlink: String,
    arch: String,
    node_mirror: String,
    npm_mirror: String,
    proxy: String,
    original_path: String,
    original_version: String,
    verify_ssl: bool,
    version: String,
    ctx: Context,
}

impl Environment {
    #[cfg(target_os = "windows")]
    fn new() -> Environment {
        let root = env::var("NVM_HOME");
        if root.is_err() {
            println!("Please set env NVM_HOME");
            exit(0);
        }

        let symlink = env::var("NVM_SYMLINK");
        if symlink.is_err() {
            println!("Please set env NVM_SYMLINK");
            exit(0);
        }

        Environment {
            settings: format!("{}{}settings.txt", root.unwrap(), MAIN_SEPARATOR),
            root: "".to_string(),
            symlink: symlink.unwrap(),
            arch: std::env::consts::ARCH.to_string(),
            node_mirror: "".to_string(),
            npm_mirror: "".to_string(),
            proxy: "none".to_string(),
            original_path: "".to_string(),
            original_version: "".to_string(),
            verify_ssl: true,
            ctx: Context::new(),
            version: "1.0".to_string(),
        }
    }

    #[cfg(target_os = "linux")]
    fn new() -> Environment {
        let exec_path = file::get_executable_path();
        if exec_path.is_err() {
            println!(
                "exec file path read fail, err:{:?}",
                exec_path.as_ref().err().unwrap()
            );
        }
        let mut settings_dir = exec_path.unwrap();
        settings_dir.pop();

        Environment {
            settings: format!(
                "{}{}settings.txt",
                settings_dir.to_str().unwrap(),
                MAIN_SEPARATOR
            ),
            root: "".to_string(),
            symlink: "".to_string(),
            arch: std::env::consts::ARCH.to_string(),
            node_mirror: "".to_string(),
            npm_mirror: "".to_string(),
            proxy: "none".to_string(),
            original_path: "".to_string(),
            original_version: "".to_string(),
            verify_ssl: true,
            ctx: Context::new(),
            version: "1.0".to_string(),
        }
    }

    #[cfg(target_os = "macos")]
    fn new() -> Environment {
        let exec_path = file::get_executable_path();
        if exec_path.is_err() {
            println!(
                "exec file path read fail, err:{:?}",
                exec_path.as_ref().err().unwrap()
            );
        }
        let mut settings_dir = exec_path.unwrap();
        settings_dir.pop();

        Environment {
            settings: format!(
                "{}{}settings.txt",
                settings_dir.to_str().unwrap(),
                MAIN_SEPARATOR
            ),
            root: "".to_string(),
            symlink: "".to_string(),
            arch: std::env::consts::ARCH.to_string(),
            node_mirror: "".to_string(),
            npm_mirror: "".to_string(),
            proxy: "none".to_string(),
            original_path: "".to_string(),
            original_version: "".to_string(),
            verify_ssl: true,
            ctx: Context::new(),
            version: "1.0".to_string(),
        }
    }
}

fn main() {
    // 设置日志
    let mut nvm_env: Environment = Environment::new();
    let args: Vec<String> = env::args().collect();
    let mut detail = String::new();
    let mut proc_arch = arch::validate(&nvm_env.arch);

    #[cfg(target_os = "windows")]
    if !Environment::is_terminal() {
        Environment::alert(
            "NVM for Windows should be run from a terminal such as CMD or PowerShell.",
            vec!["Terminal Only"],
        );
        exit(0);
    }

    if args.len() > 2 {
        detail = args[2].clone();
    }

    if args.len() > 3 {
        if args[3].eq("32") || args[3].eq("64") {
            proc_arch = args[3].to_string();
        }
    }
    if args.len() < 2 {
        help();
        return;
    }

    if args[1] != "version"
        && args[1] != "--version"
        && args[1] != "v"
        && args[1] != "-v"
        && args[1] != "--v"
    {
        nvm_env.setup();
    }

    let cmd = &args[1];
    if cmd.is_empty() {
        help();
        return;
    }
    let reload = vec![];
    match cmd.as_str() {
        "install" => nvm_env.install(&detail, &proc_arch),
        "uninstall" => nvm_env.uninstall(&detail),
        "switch" => nvm_env.switch(&detail),
        "sw" =>nvm_env.switch(&detail),
        "use" => nvm_env.use_node(&detail, &proc_arch, &reload),
        "list" => nvm_env.list(&detail),
        "ls" => nvm_env.list(&detail),
        "on" => nvm_env.enable(),
        "off" => nvm_env.disable(),
        "root" => {
            if args.len() == 3 {
                nvm_env.update_root_dir(&args[2]);
            } else {
                println!("\nCurrent Root: {}", &nvm_env.root);
            }
        }
        "v" => println!("{}", &nvm_env.version),
        "--version" => println!("{}", &nvm_env.version),
        "version" => println!("{}", &nvm_env.version),
        "--v" => println!("{}", &nvm_env.version),
        "-v" => println!("{}", &nvm_env.version),
        "arch" => {
            let trim_c: &[_] = &['\r', '\n', ' '];
            detail = detail.trim_matches(trim_c).to_string();
            if !detail.is_empty() {
                if detail != "32" && detail != "64" {
                    println!("\"{}\" is an invalid architecture. Use 32 or 64.", detail);
                    return;
                }

                nvm_env.arch = detail;
                nvm_env.save_settings();
                println!("Detault architecture set to {}-bit", nvm_env.arch);
            }
            let (_, a) = node::get_current_version();
            println!("System Default: {}-bit.", nvm_env.arch);
            println!("Currently Configured: {}-bit.", a);
        }
        "proxy" => {
            if detail == "" {
                println!("Current proxy: {}", nvm_env.proxy);
            } else {
                nvm_env.proxy = detail;
                nvm_env.save_settings();
            }
        }
        "current" => {
            let (in_use, _) = node::get_current_version();
            let res_v = Version::parse(&in_use);

            if res_v.is_err() {
                println!("{}", in_use);
            } else if in_use == "Unknown" {
                println!("No current version. Run 'nvm use x.x.x' to set a version.");
            } else {
                println!("v{}", in_use);
            }
        }
        "node_mirror" => nvm_env.set_node_mirror(&detail),
        "npm_mirror" => nvm_env.set_npm_mirror(&detail),
        _ => help(),
    }
}

fn help() {
    println!("\nRunning version 1.0 .");
    println!("\nUsage:");
    println!(" ");
    println!("  nvm arch                     : Show if node is running in 32 or 64 bit mode.");
    println!("  nvm current                  : Display active version.");
    println!("  nvm debug                    : Check the NVM4W process for known problems (troubleshooter).");
    println!("  nvm install <version> [arch] : The version can be a specific version, \"latest\" for the latest current version, or \"lts\" for the");
    println!("                                              most recent LTS version. Optionally specify whether to install the 32 or 64 bit version (defaults");
    println!("                                              to system arch). Set [arch] to \"all\" to install 32 AND 64 bit versions.");
    println!("                                              Add --insecure to the end of this command to bypass SSL validation of the remote download server.");
    println!("  nvm list [available]         : List the node.js installations. Type \"available\" at the end to see what can be installed. Aliased as ls.");
    println!("  nvm on                       : Enable node.js version management.");
    println!("  nvm off                      : Disable node.js version management.");
    println!("  nvm proxy [url]              : Set a proxy to use for downloads. Leave [url] blank to see the current proxy.");
    println!("                                               Set [url] to \"none\" to remove the proxy.");
    println!("  nvm node_mirror [url]        : Set the node mirror. Defaults to https://nodejs.org/dist/. Leave [url] blank to use default url.");
    println!("  nvm npm_mirror [url]         : Set the npm mirror. Defaults to https://github.com/npm/cli/archive/. Leave [url] blank to default url.");
    println!("  nvm uninstall <version>      : The version must be a specific version.");
    println!("  nvm switch [arch]            : Switch to use already install version. Optionally specify 32/64bit architecture. Aliased as sw.");
    println!("  nvm use [version] [arch]     : Switch to use the specified version. Optionally use \"latest\", \"lts\", or \"newest\".");
    println!("                                              \"newest\" is the latest installed version. Optionally specify 32/64bit architecture.");
    println!("                                              nvm use <arch> will continue using the selected version, but switch to 32/64 bit mode.");
    println!("  nvm root [path]              : Set the directory where nvm should store different versions of node.js.");
    println!("                                              If <path> is not set, the current root will be displayed.");
    println!("  nvm [--]version              : Displays the current running version of nvm for Windows. Aliased as v.");
    println!(" ");
}

impl Environment {
    fn set_node_mirror(&mut self, url: &str) {
        self.node_mirror = url.to_string();
        self.save_settings();
    }

    fn set_npm_mirror(&mut self, url: &str) {
        self.npm_mirror = url.to_string();
        self.save_settings();
    }

    fn save_settings(&mut self) {
        let mut content = String::new();
        content.push_str(format!("root: {}\r\n", self.root.trim_end_matches("\r\n")).as_str());
        content.push_str(format!("arch: {}\r\n", self.arch.trim_end_matches("\r\n")).as_str());
        content.push_str(format!("proxy: {}\r\n", self.proxy.trim_end_matches("\r\n")).as_str());
        content.push_str(
            format!(
                "original_path: {}\r\n",
                self.original_path.trim_end_matches("\r\n")
            )
            .as_str(),
        );
        content.push_str(
            format!(
                "original_version: {}\r\n",
                self.original_version.trim_end_matches("\r\n")
            )
            .as_str(),
        );
        content.push_str(
            format!(
                "node_mirror: {}\r\n",
                self.node_mirror.trim_end_matches("\r\n")
            )
            .as_str(),
        );
        content.push_str(
            format!(
                "npm_mirror: {}\r\n",
                self.npm_mirror.trim_end_matches("\r\n")
            )
            .as_str(),
        );
        if let Err(err) = fs::write(self.settings.as_str(), content.as_bytes()) {
            println!("Save setting fail,err:{}", err)
        }
    }

    #[cfg(target_os = "windows")]
    fn alert(msg: &str, caption: Vec<&str>) {
        unsafe {
            // 获取前台窗口句柄
            let hand: HWND = GetForegroundWindow();

            // 默认标题
            let mut title = "title";
            if caption.len() > 0 {
                title = caption.get(0).unwrap();
            }

            // 将字符串转换为宽字符串指针
            let msg_wide: Vec<u16> = msg.encode_utf16().chain(Some(0)).collect();
            let title_wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();

            // 调用 MessageBoxW 函数
            MessageBoxW(
                hand,
                PCWSTR(msg_wide.as_ptr()),
                PCWSTR(title_wide.as_ptr()),
                MB_OK,
            );
        }
    }

    #[cfg(target_os = "windows")]
    fn is_terminal() -> bool {
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                return false;
            }

            let mut mode = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                return false;
            }
        }
        true
    }

    fn get_version(
        &self,
        version: &str,
        cpu_arch: &str,
        local_install_only: &Vec<bool>,
    ) -> Result<(String, String), String> {
        let mut requested_version = version.to_string();
        let mut version = version.to_string();
        let mut cpu_arch = cpu_arch.to_string();
        requested_version = requested_version.to_uppercase();

        if cpu_arch != "" {
            if cpu_arch != "32" && cpu_arch != "64" && cpu_arch != "all" {
                return Err(format!(
                    "{} is not a valid CPU architecture. Must be 32 or 64.",
                    cpu_arch
                ));
            }
        } else {
            cpu_arch = self.arch.to_string();
        }

        if cpu_arch != "all" {
            cpu_arch = arch::validate(&cpu_arch);
        }

        if version == "" {
            return Err("A version argument is required but missing.".to_string());
        }

        if version == "latest" || version == "node" {
            let res = node::get_latest(&self.ctx.web_ctx);
            if res.is_none() {
                return Err("latest missing.".to_string());
            }
            version = res.unwrap();
            println!("{}", version);
        }

        if version == "lts" {
            let res = node::get_lts(&self.ctx.web_ctx);
            if res.is_none() {
                return Err("lts missing.".to_string());
            }
            version = res.unwrap();
            println!("{}", version);
        }

        if version == "newest" {
            let installed = node::get_installed(&self.root);
            if installed.len() == 0 {
                return Err("No versions of node.js found. Try installing the latest by typing nvm install latest.".to_string());
            }
            version = installed.get(0).unwrap().to_string();
        }

        if version == "32" || version == "64" {
            cpu_arch = version.to_string();

            let (v, _) = node::get_current_version();
            version = v;
        }

        let mut version = self.version_number_from(version.as_str());
        let v = semver::Version::parse(&version);
        match v {
            Ok(_v) => {
                let sv: Vec<&str> = version.split('.').collect();
                if sv.len() < 3 {
                    version = self.find_latest_sub_version(version.as_str(), &vec![]);
                } else {
                    version = self.clean_version(version.as_str());
                }

                version = self.version_number_from(version.as_str());
            }
            Err(err) => {
                let err_str = err.to_string();
                if err_str.contains("No Major.Minor.Patch") {
                    let mut latest_local_install = false;
                    if local_install_only.len() > 0 {
                        latest_local_install = local_install_only.get(0).unwrap().clone();
                    }
                    let tmp = vec![latest_local_install];
                    version = self.find_latest_sub_version(version.as_str(), &tmp);
                    if version.len() == 0 {
                        let tmp = format!("Unrecognized version: \"{}\"", requested_version);
                        return Err(tmp);
                    }
                }
            }
        }
        Ok((version, cpu_arch))
    }

    #[cfg(target_os = "windows")]
    fn install(&mut self, version: &str, cpu_arch: &str) {
        let requested_version = version.to_string();
        let args: Vec<String> = env::args().collect();
        let last_arg = args.get(args.len() - 1).unwrap().clone();
        if last_arg == "--insecure" {
            self.verify_ssl = false;
        }

        if version.starts_with("--") {
            println!("\"--\" prefixes are unnecessary in NVM for Windows!");
            let version = version.replace("-", "");
            println!("attempting to install \"{}\" instead...\n\n", version);
            sleep(Duration::from_secs(2));
        }

        let v_res = self.get_version(version, cpu_arch, &vec![]);
        if v_res.is_err() {
            let err = v_res.as_ref().err().unwrap();
            if err.contains("No major.Minor.Patch") {
                let sv_res = semver::Version::parse(version);
                if sv_res.is_err() {
                    let version = self.find_latest_sub_version(version, &vec![]);
                    if version.len() == 0 {
                        println!("Unrecognized version: \"{}\"", requested_version);
                        if version == "" {
                            println!(" ");
                            help();
                        }
                        return;
                    }
                }
            }
        }
        if v_res.is_err() {
            println!("\"{}\" is not a valid version.", version);
            println!("Please use a valid semantic version number, \"lts\", or \"latest\".");
            return;
        }

        if self.check_version_exceeds_latest(version) {
            println!(
                "Node.js v{} is not yet released or is not available.",
                version
            );
            return;
        }

        if cpu_arch == "64" && !WebContext::is_node64_bid_available(version) {
            println!("Node.js v{} is only available in 32-bit.", version);
            return;
        }

        if !node::is_version_installed(&self.root, version, cpu_arch) {
            if !node::is_version_available(version, &self.ctx.web_ctx) {
                let url: String = self.ctx.web_ctx.get_full_node_url("index.json");
                println!("\nVersion {} is not available. \n\nThe complete list of available version can be fount at {}", version, url);
                return;
            }

            // Make the output directories
            // node version dir
            let mut node_dir_path = PathBuf::from(&self.root);
            let version_name = format!("v{}", version);
            node_dir_path.push(&version_name);
            fs::create_dir_all(&node_dir_path).unwrap();

            // node_modules
            node_dir_path.push("node_modules");
            fs::create_dir_all(&node_dir_path).unwrap();

            if !self.verify_ssl {
                println!("\nWARNING: The remote SSL certificate will not be validated during the download process.\n");
            }

            // Download node todo 疑惑点
            let append32 = node::is_version_installed(&self.root, version, "64");
            let append64 = node::is_version_installed(&self.root, version, "32");
            if (cpu_arch == "32" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "32")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "32", append32);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 32-bit executable.", version);
                    return;
                }
            }
            if (cpu_arch == "64" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "64")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "64", append64);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 64-bit executable.", version);
                    return;
                }
            }

            node_dir_path.push("npm");
            if file::exists(node_dir_path.to_str().unwrap()) {
                let npm_v = node::get_npm_version(version, &self.ctx.web_ctx);
                println!("npm v{} installed successfully.", npm_v);
                println!("\n\n Installation complete. If you want to use this version, type\n\n nvm use {}", version);
                return;
            }
            let npm_v = node::get_npm_version(version, &self.ctx.web_ctx);
            let success = self.ctx.web_ctx.get_npm(&self.root, &npm_v);
            if success {
                println!("Installing npm v {}...", version);

                let temp_dir = filepath::join(&self.root, vec!["temp"]);
                let source_name = format!("npm-v{}.zip", npm_v);
                let target_name = "nvm-npm";
                let source_path = filepath::join(&temp_dir, vec![&source_name]);
                let target_path = filepath::join(&temp_dir, vec![target_name]);

                if let Err(err) = file::unzip(&source_path, &target_path, false) {
                    fs::remove_dir_all(&node_dir_path).unwrap_or_else(|e| {
                        println!(
                            "Failed to remove directory where {} and err {}",
                            node_dir_path.to_string_lossy().to_string(),
                            e
                        );
                    });
                    println!("Could not download npm {}: {}", npm_v, err);
                    return;
                }

                let mut temp_npm_bin = filepath::join(
                    &temp_dir,
                    vec!["nvm-npm", format!("cli-{}", npm_v).as_str(), "bin"],
                );
                if !file::exists(&temp_npm_bin) {
                    temp_npm_bin = filepath::join(
                        &temp_dir,
                        vec!["nvm-npm", format!("npm-{}", npm_v).as_str(), "bin"],
                    );
                }

                if !file::exists(&temp_npm_bin) {
                    println!("Failed to extract npm. Count not find {}", temp_npm_bin);
                    exit(0);
                }

                let _ = fs::rename(
                    filepath::join(&temp_npm_bin, vec!["npm"]),
                    filepath::join(&self.root, vec![format!("v{}", version).as_str(), "npm"]),
                );
                let _ = fs::rename(
                    filepath::join(&temp_npm_bin, vec!["npm.cmd"]),
                    filepath::join(
                        self.root.as_str(),
                        vec![format!("v{}", version).as_str(), "npm.cmd"],
                    ),
                );

                if file::exists(filepath::join(&temp_npm_bin, vec!["npx"]).as_str()) {
                    let _ = fs::rename(
                        filepath::join(&temp_npm_bin, vec!["npx"]),
                        filepath::join(
                            self.root.as_str(),
                            vec![format!("v{}", version).as_str(), "npx"],
                        ),
                    );
                    let _ = fs::rename(
                        filepath::join(&temp_npm_bin, vec!["npx.cmd"]),
                        filepath::join(
                            self.root.as_str(),
                            vec![format!("v{}", version).as_str(), "npx.cmd"],
                        ),
                    );
                }

                let mut npm_source_path = filepath::join(
                    &temp_dir,
                    vec!["nvm-npm", format!("npm-{}", npm_v).as_str()],
                );
                if !file::exists(&npm_source_path) {
                    npm_source_path = filepath::join(
                        &temp_dir,
                        vec!["nvm-npm", format!("cli-{}", npm_v).as_str()],
                    );
                }

                let move_npm_path = filepath::join(
                    &self.root,
                    vec![format!("v{}", version).as_str(), "node_modules", "npm"],
                );
                let mut move_npm_err = fs::rename(&npm_source_path, &move_npm_path);
                if move_npm_err.is_err() {
                    for i in [1, 2, 3, 8, 16] {
                        sleep(Duration::from_secs(i as u64));
                        move_npm_err = fs::rename(&npm_source_path, &move_npm_path);
                        if move_npm_err.is_ok() {
                            break;
                        } else {
                            println!(
                                "Error: move {} to {} fail,err:{}",
                                &npm_source_path,
                                &move_npm_path,
                                move_npm_err.as_ref().err().unwrap()
                            );
                        }
                    }
                }

                if move_npm_err.is_ok() {
                    fs::remove_dir_all(&temp_dir).unwrap();
                    println!("\n\n Installation complete. If you want to use this version, type\n\n nvm use {}",version);
                } else if move_npm_err.is_err() {
                    println!("Error: Unable to move directory {}", npm_source_path);
                } else {
                    println!(
                        "Error: Unable to install NPM: {}",
                        move_npm_err.as_ref().err().unwrap()
                    );
                }
            } else {
                println!("Could not download npm for node v{}", version);
                println!(
                    "Please visit https://github.com/npm/cli/releases/tag/v{} to download npm.",
                    npm_v
                );
                println!("It should be extracted to {} \\v{}", &self.root, version);
            }
            self.verify_ssl = true;
            return;
        } else {
            println!("Version {} is already installed.", version);
            return;
        }
    }

    #[cfg(target_os = "linux")]
    fn install(&mut self, version: &str, cpu_arch: &str) {
        let requested_version = version.to_string();
        let args: Vec<String> = env::args().collect();
        let last_arg = args.get(args.len() - 1).unwrap().clone();
        if last_arg == "--insecure" {
            self.verify_ssl = false;
        }

        if version.starts_with("--") {
            println!("\"--\" prefixes are unnecessary in NVM for Windows!");
            let version = version.replace("-", "");
            println!("attempting to install \"{}\" instead...\n\n", version);
            sleep(Duration::from_secs(2));
        }

        let v_res = self.get_version(version, cpu_arch, &vec![]);
        if v_res.is_err() {
            let err = v_res.as_ref().err().unwrap();
            if err.contains("No major.Minor.Patch") {
                let sv_res = semver::Version::parse(version);
                if sv_res.is_err() {
                    let version = self.find_latest_sub_version(version, &vec![]);
                    if version.len() == 0 {
                        println!("Unrecognized version: \"{}\"", requested_version);
                        if version == "" {
                            println!(" ");
                            help();
                        }
                        return;
                    }
                }
            }
        }
        if v_res.is_err() {
            println!("\"{}\" is not a valid version.", version);
            println!("Please use a valid semantic version number, \"lts\", or \"latest\".");
            return;
        }

        if self.check_version_exceeds_latest(version) {
            println!(
                "Node.js v{} is not yet released or is not available.",
                version
            );
            return;
        }

        if cpu_arch == "64" && !WebContext::is_node64_bid_available(version) {
            println!("Node.js v{} is only available in 32-bit.", version);
            return;
        }

        if !node::is_version_installed(&self.root, version, cpu_arch) {
            if !node::is_version_available(version, &self.ctx.web_ctx) {
                let url: String = self.ctx.web_ctx.get_full_node_url("index.json");
                println!("\nVersion {} is not available. \n\nThe complete list of available version can be fount at {}", version, url);
                return;
            }

            // Make the output directories
            // node version dir
            let mut node_dir_path = PathBuf::from(&self.root);
            let version_name = format!("v{}", version);
            node_dir_path.push(&version_name);
            fs::create_dir_all(&node_dir_path).unwrap();

            // node_modules
            node_dir_path.push("lib");
            node_dir_path.push("node_modules");
            fs::create_dir_all(&node_dir_path).unwrap();

            if !self.verify_ssl {
                println!("\nWARNING: The remote SSL certificate will not be validated during the download process.\n");
            }

            // Download node todo 疑惑点
            let append32 = node::is_version_installed(&self.root, version, "64");
            let append64 = node::is_version_installed(&self.root, version, "32");
            if (cpu_arch == "32" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "32")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "32", append32);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 32-bit executable.", version);
                    return;
                }
            }
            if (cpu_arch == "64" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "64")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "64", append64);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 64-bit executable.", version);
                    return;
                }
            }
            node_dir_path.push("npm");

            if file::exists(node_dir_path.to_str().unwrap()) {
                let npm_v = node::get_npm_version(version, &self.ctx.web_ctx);
                println!("npm v{} installed successfully.", npm_v);
                println!("\n\n Installation complete. If you want to use this version, type\n\n nvm use {}", version);
                return;
            } else {
                // 不再支持
                println!("Node versions that are not bound to npm are no longer supported");
            }
            self.verify_ssl = true;
            return;
        } else {
            println!("Version {} is already installed.", version);
            return;
        }
    }

    #[cfg(target_os = "macos")]
    fn install(&mut self, version: &str, cpu_arch: &str) {
        let requested_version = version.to_string();
        let args: Vec<String> = env::args().collect();
        let last_arg = args.get(args.len() - 1).unwrap().clone();
        if last_arg == "--insecure" {
            self.verify_ssl = false;
        }

        if version.starts_with("--") {
            println!("\"--\" prefixes are unnecessary in NVM for Windows!");
            let version = version.replace("-", "");
            println!("attempting to install \"{}\" instead...\n\n", version);
            sleep(Duration::from_secs(2));
        }

        let v_res = self.get_version(version, cpu_arch, &vec![]);
        if v_res.is_err() {
            let err = v_res.as_ref().err().unwrap();
            if err.contains("No major.Minor.Patch") {
                let sv_res = semver::Version::parse(version);
                if sv_res.is_err() {
                    let version = self.find_latest_sub_version(version, &vec![]);
                    if version.len() == 0 {
                        println!("Unrecognized version: \"{}\"", requested_version);
                        if version == "" {
                            println!(" ");
                            help();
                        }
                        return;
                    }
                }
            }
        }
        if v_res.is_err() {
            println!("\"{}\" is not a valid version.", version);
            println!("Please use a valid semantic version number, \"lts\", or \"latest\".");
            return;
        }

        if self.check_version_exceeds_latest(version) {
            println!(
                "Node.js v{} is not yet released or is not available.",
                version
            );
            return;
        }

        if cpu_arch == "64" && !WebContext::is_node64_bid_available(version) {
            println!("Node.js v{} is only available in 32-bit.", version);
            return;
        }

        if !node::is_version_installed(&self.root, version, cpu_arch) {
            if !node::is_version_available(version, &self.ctx.web_ctx) {
                let url: String = self.ctx.web_ctx.get_full_node_url("index.json");
                println!("\nVersion {} is not available. \n\nThe complete list of available version can be fount at {}", version, url);
                return;
            }

            // Make the output directories
            // node version dir
            let mut node_dir_path = PathBuf::from(&self.root);
            let version_name = format!("v{}", version);
            node_dir_path.push(&version_name);
            fs::create_dir_all(&node_dir_path).unwrap();

            // node_modules
            node_dir_path.push("lib");
            node_dir_path.push("node_modules");
            fs::create_dir_all(&node_dir_path).unwrap();

            if !self.verify_ssl {
                println!("\nWARNING: The remote SSL certificate will not be validated during the download process.\n");
            }

            // Download node todo 疑惑点
            let append32 = node::is_version_installed(&self.root, version, "64");
            let append64 = node::is_version_installed(&self.root, version, "32");
            if (cpu_arch == "32" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "32")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "32", append32);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 32-bit executable.", version);
                    return;
                }
            }
            if (cpu_arch == "64" || cpu_arch == "all")
                && !node::is_version_installed(&self.root, version, "64")
            {
                let success = self
                    .ctx
                    .web_ctx
                    .get_node_js(&self.root, version, "64", append64);
                if !success {
                    fs::remove_dir_all(&node_dir_path).unwrap();
                    println!("Could not download node.js v{} 64-bit executable.", version);
                    return;
                }
            }
            node_dir_path.push("npm");

            if file::exists(node_dir_path.to_str().unwrap()) {
                let npm_v = node::get_npm_version(version, &self.ctx.web_ctx);
                println!("npm v{} installed successfully.", npm_v);
                println!("\n\n Installation complete. If you want to use this version, type\n\n nvm use {}", version);
                return;
            } else {
                // 不再支持
                println!("Node versions that are not bound to npm are no longer supported");
            }
            self.verify_ssl = true;
            return;
        } else {
            println!("Version {} is already installed.", version);
            return;
        }
    }

    fn uninstall(&self, version: &str) {
        let mut v: String = version.to_string();
        if v.len() == 0 {
            println!("Provide the version you want to uninstall.");
            help();
            return;
        }

        if v.to_lowercase() == "latest" || v.to_lowercase() == "node" {
            v = get_latest(&self.ctx.web_ctx).unwrap();
        } else if v.to_lowercase() == "lts" {
            v = get_lts(&self.ctx.web_ctx).unwrap();
        } else if v.to_lowercase() == "newest" {
            let installed = node::get_installed(&self.root);
            if installed.len() == 0 {
                println!("No version of node.js found. Try installing the latest by typing nvm install latest.");
                return;
            }
            v = installed[0].clone();
        }

        v = self.clean_version(&v);
        if node::is_version_installed(&self.root, &v, "32")
            || node::is_version_installed(&self.root, &v, "64")
        {
            println!("Uninstalling node v{}...", &v);
            let (cv, _) = node::get_current_version();
            if cv == v {
                let arg = filepath::clean(&self.symlink);
                let res = cmd::elevated_run(&self.root, "rmdir", vec![&arg]);
                if res.is_err() {
                    println!("elevated_run fail,err:{}", res.as_ref().err().unwrap());
                    return;
                }
            }
            let remove_path = filepath::join(&self.root, vec![&format!("v{}", v)]);
            let e = fs::remove_dir_all(&remove_path);
            if e.is_err() {
                println!("Error removing node v{}", v);
                println!(
                    "Manually remove {}.",
                    filepath::join(&self.root, vec![&format!("v{}", v)])
                );
            } else {
                println!("done");
            }
        } else {
            println!(
                "node v{} is not installed. Type \"nvm list\" to see what is installed",
                v
            );
        }
        return;
    }

    fn check_version_exceeds_latest(&self, version: &str) -> bool {
        let url = self.ctx.web_ctx.get_full_node_url("latest/SHASUMS256.txt");
        let content = self.ctx.web_ctx.get_remote_text_file(&url);
        let re = regex::Regex::new("node-v(.+)+msi").unwrap();
        let reg = Regex::new("node-v|-[xa].+").unwrap();
        let temp = re.find(&content).unwrap().as_str();
        let latest = reg.replace_all(temp, "");
        let v_arr: Vec<&str> = version.split(".").collect();
        let l_arr: Vec<&str> = latest.split(".").collect();
        for i in 0..l_arr.len() {
            let lat = l_arr[i].parse::<i32>().unwrap();
            let ver = v_arr[i].parse::<i32>().unwrap();
            if ver < lat {
                return false;
            } else if ver > lat {
                return true;
            }
        }
        return false;
    }

    fn clean_version(&self, version: &str) -> String {
        let mut re = Regex::new("\\d+.\\d+.\\d+").unwrap();
        let mut matched = re.find(version).unwrap().as_str().to_string();
        if matched.len() == 0 {
            re = Regex::new("\\d+.\\d+").unwrap();
            matched = re.find(version).unwrap().as_str().to_string();
            if matched.len() == 0 {
                matched = format!("{}.0.0", version);
            } else {
                matched = format!("{}.0", version);
            }
            println!("{}", matched);
        }
        matched.to_string()
    }

    fn update_root_dir(&mut self, path: &str) {
        if !file::exists(path) {
            println!("{} does not exist or could not be found.", path);
            return;
        }

        let current_root = self.root.clone();
        self.root = filepath::clean(path);

        let cmd_source = format!("{}/elevate.cmd", current_root);
        let cmd_target = format!("{}/elevate.cmd", self.root);
        let hard_link = fs::hard_link(filepath::clean(&cmd_source), filepath::clean(&cmd_target));
        if hard_link.is_err() {
            println!("Update root dir fail,err:{}", hard_link.err().unwrap());
            return;
        }

        let vbs_source = format!("{}/elevate.vbs", current_root);
        let vbs_target = format!("{}/elevate.vbs", self.root);
        let hard_link = fs::hard_link(filepath::clean(&vbs_source), filepath::clean(&vbs_target));
        if hard_link.is_err() {
            println!("Update root dir fail,err:{}", hard_link.err().unwrap());
            return;
        }

        self.save_settings();

        if current_root != self.root {
            println!("\n Root has been change from {} to {}", current_root, path);
        }
    }

    fn version_number_from(&self, version: &str) -> String {
        let re = Regex::new("[^0-9]").unwrap();

        if re.is_match(&version[0..version.len() - 2]) {
            if &version[0..1] == "v" {
                let path = format!("latest-{}/SHASUMS256.txt", version);
                let url = self.ctx.web_ctx.get_full_node_url(path.as_str());
                let tmp = self.ctx.web_ctx.get_remote_text_file(&url);
                let versions: Vec<&str> = tmp.split("\n").collect();
                let content = versions.get(0).unwrap();
                if content.contains("node") {
                    let parts: Vec<&str> = content.split("-").collect();
                    if parts.len() > 1 {
                        if &parts[1][0..1] == "v" {
                            return parts[1][1..parts.len()].to_string();
                        }
                    }
                }
                println!("\"{}\" is not a valid version or known alias. \n", version);
                println!("\n Available aliases: latest, node(latest), lts\nNamed releases (boron, dubnium, etc) are also supported.");
                exit(0);
            }
        }
        let mut tmp = version.to_string();
        while re.is_match(&tmp[0..1]) {
            tmp = tmp[1..tmp.len()].to_owned();
        }

        return tmp;
    }

    fn split_version(&self, version: &str) -> HashMap<&str, i32> {
        let parts: Vec<&str> = version.split(',').collect();
        let mut result: HashMap<&str, i32> = HashMap::with_capacity(3);

        let keys = ["major", "minor", "patch"];

        for (i, &part) in parts.iter().enumerate() {
            let v = part.parse::<i32>().unwrap();
            result.insert(keys[i], v);
        }

        result
    }

    fn find_latest_sub_version(&self, version: &str, local_only: &Vec<bool>) -> String {
        if local_only.len() > 0 && *local_only.get(0).unwrap() {
            let installed = node::get_installed(&self.root);
            let mut result = String::from("");
            for v in installed.into_iter() {
                let prefix = format!("v{}", version);
                if v.starts_with(&prefix) {
                    if result != "" {
                        let current = semver::Version::parse(
                            self.version_number_from(result.as_str()).as_str(),
                        )
                        .unwrap();
                        let next =
                            semver::Version::parse(self.version_number_from(v.as_str()).as_str())
                                .unwrap();

                        if current.lt(&next) {
                            result = v.clone();
                        }
                    } else {
                        result = v.clone();
                    }
                }
            }

            if result.trim().len() > 0 {
                return self.version_number_from(result.as_str());
            }
        }
        let tmp: Vec<&str> = version.split(",").collect();
        if tmp.len() == 2 {
            let (all, _, _, _, _, _) = node::get_available(&self.ctx.web_ctx);
            let tmp = version.to_owned() + ".0";
            let mut requested = self.split_version(tmp.as_str());

            for v in all {
                let available = self.split_version(v.to_string().as_str());
                if requested.get("major").unwrap() == available.get("major").unwrap() {
                    if requested.get("minor").unwrap() == available.get("minor").unwrap() {
                        if available.get("patch").unwrap() == requested.get("patch").unwrap() {
                            requested.insert("patch", available.get("patch").unwrap().clone());
                        }
                    }
                    if requested.get("minor").unwrap() > available.get("minor").unwrap() {
                        break;
                    }
                }

                if requested.get("major").unwrap() > available.get("major").unwrap() {
                    break;
                }
            }
            return format!(
                "{}.{}.{}",
                requested.get("major").unwrap(),
                requested.get("minor").unwrap(),
                requested.get("patch").unwrap()
            );
        }
        let path = format!("latest-v{}.x/SHASUMS256.txt", version);
        let url = self.ctx.web_ctx.get_full_node_url(path.as_str());
        let content = self.ctx.web_ctx.get_remote_text_file(url.as_str());
        let re = Regex::new("node-v(.+)+msi").unwrap();
        let reg = Regex::new("node-v|-[xa].+").unwrap();
        let find_str = re.find(&content).unwrap().as_str();
        let latest = reg.replace_all(find_str, "");
        latest.to_string()
    }

    fn access_denied(err: &str) -> bool {
        println!("{}", err);

        if err.to_ascii_lowercase().contains("access is denied") {
            println!("See https://bit.ly/nvm4w-help");
            return true;
        }
        return false;
    }

    // 切换node版本
    fn switch(&self, arch: &str) {
        let (in_use, _) = node::get_current_version();
        let mut installed_versions = node::get_installed(&self.root);

        if installed_versions.is_empty() {
            println!("No installations recognized.");
            return;
        }

        installed_versions
            .sort_by(|a, b| Version::parse(&b[1..]).unwrap().cmp(&Version::parse(&a[1..]).unwrap()));

        let mut i = 0;
        for version in &installed_versions {
            if version.contains("v") {
                let v_in_use = format!("v{}", in_use);
                if v_in_use == *version {
                    break;
                }
                i += 1;
            }
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick node, hint it might be on the second page")
            .default(i)
            .max_length(5)
            .items(&installed_versions[..])
            .interact()
            .unwrap();

        if selection != i {
            let reload = vec![];
            self.use_node(&installed_versions[selection][1..], arch, &reload)
        } else {
            println!(
                "Now using node v{} ({}-bit)",
                &installed_versions[selection], arch
            );
        }
    }

    #[cfg(target_os = "windows")]
    fn use_node(&self, version: &str, cpu_arch: &str, reload: &Vec<bool>) {
        let local_install_only = vec![true];
        let res = self.get_version(version, cpu_arch, &local_install_only);

        if res.is_err() {
            let err = res.as_ref().err().unwrap();
            if err.contains("No Major.Minor.Patch") {
                println!("{}", err);
                return;
            }
        }
        let (version, cpu_arch) = res.unwrap();

        if !node::is_version_installed(&self.root, &version, &cpu_arch) {
            println!("node v{} ({}-bit) is not installed.", &version, &cpu_arch);
            if cpu_arch == "32" {
                if node::is_version_installed(&self.root, &version, "64") {
                    println!("\n Did you mean node v{} (64-bit)?\n If so, type \"nvm use {} 64\" to use it.",&version,&version);
                }
            }
            if cpu_arch == "64" {
                if node::is_version_installed(&self.root, &version, &cpu_arch) {
                    println!("\n Did you mean node v{} (32-bit)?\n If so, type \"nvm use {} 32\" to use it.",&version,&version);
                }
            }
            return;
        }

        let symlink = filepath::clean(&self.symlink);
        // Remove symlink if it already exists
        if file::exists(&self.symlink) {
            let res = cmd::elevated_run(&self.root, "rmdir", vec![&symlink]);
            if res.is_err() {
                if Self::access_denied(res.as_ref().err().unwrap()) {
                    return;
                }
            }
        }

        let mut ok = true;
        let v_version = format!("v{}", version);
        let target_symlink = filepath::join(&self.root, vec![&v_version]);
        let res = cmd::elevated_run(&self.root, "mklink", vec!["/D", &symlink, &target_symlink]);

        if res.is_err() {
            let err = res.as_ref().err().unwrap();
            if err.contains("not have sufficient privilege")
                || err.to_lowercase().contains("access is denied")
            {
                let res =
                    cmd::elevated_run(&self.root, "mklink", vec!["/D", &symlink, &target_symlink]);

                if res.is_err() {
                    ok = false;
                    println!("{}", err)
                }
            } else if err.contains("file already exists") {
                let res = cmd::elevated_run(&self.root, "rmdir", vec![&symlink]);
                let mut reloadable = true;
                if reload.len() > 0 {
                    reloadable = reload[0];
                }

                if res.is_err() {
                    println!("{}", res.as_ref().err().unwrap());
                } else if reloadable {
                    let reload = vec![false];
                    self.use_node(&version, &cpu_arch, &reload);
                    return;
                }
            } else {
                println!("{}", err);
            }
        }

        if !ok {
            return;
        }

        // Use the assigned CPu architechture
        let cpu_arch = arch::validate(&cpu_arch);
        let node_path = filepath::join(&self.root, vec![&v_version, "node.exe"]);
        let node32_path = filepath::join(&self.root, vec![&v_version, "node32.exe"]);
        let node64_path = filepath::join(&self.root, vec![&v_version, "node64.exe"]);
        let node_exists = file::exists(&node_path);
        let node32_exists = file::exists(&node32_path);
        let node64_exists = file::exists(&node64_path);

        if node32_exists && cpu_arch == "32" {
            if node_exists {
                let _ = fs::rename(&node_path, &node64_path);
            }
            let _ = fs::rename(&node32_path, &node32_path);
        }
        if node64_exists && cpu_arch == "64" {
            if node_exists {
                let _ = fs::rename(&node_path, &node32_path);
            }
            let _ = fs::rename(node64_path, &node_path);
        }
        println!("Now using node v{} ({}-bit)", version, cpu_arch);
    }

    #[cfg(target_os = "linux")]
    fn use_node(&self, version: &str, cpu_arch: &str, reload: &Vec<bool>) {
        let local_install_only = vec![true];
        let res = self.get_version(version, cpu_arch, &local_install_only);

        if res.is_err() {
            let err = res.as_ref().err().unwrap();
            if err.contains("No Major.Minor.Patch") {
                println!("{}", err);
                return;
            }
        }
        let (version, cpu_arch) = res.unwrap();

        if !node::is_version_installed(&self.root, &version, &cpu_arch) {
            println!("node v{} ({}-bit) is not installed.", &version, &cpu_arch);
            if cpu_arch == "32" {
                if node::is_version_installed(&self.root, &version, "64") {
                    println!("\n Did you mean node v{} (64-bit)?\n If so, type \"nvm use {} 64\" to use it.",&version,&version);
                }
            }
            if cpu_arch == "64" {
                if node::is_version_installed(&self.root, &version, &cpu_arch) {
                    println!("\n Did you mean node v{} (32-bit)?\n If so, type \"nvm use {} 32\" to use it.",&version,&version);
                }
            }
            return;
        }

        let symlink = filepath::clean(&self.symlink);
        // Remove symlink if it already exists
        if file::exists(&self.symlink) {
            let res = fs::remove_file(&symlink);
            if res.is_err() {
                if Self::access_denied(&res.as_ref().err().unwrap().to_string()) {
                    return;
                }
            }
        }

        let mut ok = true;
        let v_version = format!("v{}", version);
        let target_symlink = filepath::join(&self.root, vec![&v_version]);
        let res = std::os::unix::fs::symlink(&target_symlink, &symlink);

        if res.is_err() {
            let err = res.as_ref().err().unwrap().to_string();
            if err.contains("not have sufficient privilege")
                || err.to_lowercase().contains("access is denied")
            {
                let res = std::os::unix::fs::symlink(&target_symlink, &symlink);

                if res.is_err() {
                    ok = false;
                    println!("{}", err)
                }
            } else if err.contains("file already exists") {
                let res = fs::remove_file(&symlink);
                let mut reloadable = true;
                if reload.len() > 0 {
                    reloadable = reload[0];
                }

                if res.is_err() {
                    println!("{}", res.as_ref().err().unwrap());
                } else if reloadable {
                    let reload = vec![false];
                    self.use_node(&version, &cpu_arch, &reload);
                    return;
                }
            } else {
                println!("{}", err);
                return;
            }
        }

        if !ok {
            return;
        }

        // Use the assigned CPu architechture
        let cpu_arch = arch::validate(&cpu_arch);
        let node_path = filepath::join(&self.root, vec![&v_version, "node.exe"]);
        let node32_path = filepath::join(&self.root, vec![&v_version, "node32.exe"]);
        let node64_path = filepath::join(&self.root, vec![&v_version, "node64.exe"]);
        let node_exists = file::exists(&node_path);
        let node32_exists = file::exists(&node32_path);
        let node64_exists = file::exists(&node64_path);

        if node32_exists && cpu_arch == "32" {
            if node_exists {
                let _ = fs::rename(&node_path, &node64_path);
            }
            let _ = fs::rename(&node32_path, &node32_path);
        }
        if node64_exists && cpu_arch == "64" {
            if node_exists {
                let _ = fs::rename(&node_path, &node32_path);
            }
            let _ = fs::rename(node64_path, &node_path);
        }
        println!("Now using node v{} ({}-bit)", version, cpu_arch);
    }

    #[cfg(target_os = "macos")]
    fn use_node(&self, version: &str, cpu_arch: &str, reload: &Vec<bool>) {
        let local_install_only = vec![true];
        let res = self.get_version(version, cpu_arch, &local_install_only);

        if res.is_err() {
            let err = res.as_ref().err().unwrap();
            if err.contains("No Major.Minor.Patch") {
                println!("{}", err);
                return;
            }
        }
        let (version, cpu_arch) = res.unwrap();

        if !node::is_version_installed(&self.root, &version, &cpu_arch) {
            println!("node v{} ({}-bit) is not installed.", &version, &cpu_arch);
            if cpu_arch == "32" {
                if node::is_version_installed(&self.root, &version, "64") {
                    println!("\n Did you mean node v{} (64-bit)?\n If so, type \"nvm use {} 64\" to use it.",&version,&version);
                }
            }
            if cpu_arch == "64" {
                if node::is_version_installed(&self.root, &version, &cpu_arch) {
                    println!("\n Did you mean node v{} (32-bit)?\n If so, type \"nvm use {} 32\" to use it.",&version,&version);
                }
            }
            return;
        }

        let symlink = filepath::clean(&self.symlink);
        // Remove symlink if it already exists
        if file::exists(&self.symlink) {
            let res = fs::remove_file(&symlink);
            if res.is_err() {
                if Self::access_denied(&res.as_ref().err().unwrap().to_string()) {
                    return;
                }
            }
        }

        let mut ok = true;
        let v_version = format!("v{}", version);
        let target_symlink = filepath::join(&self.root, vec![&v_version]);
        let res = std::os::unix::fs::symlink(&target_symlink, &symlink);

        if res.is_err() {
            let err = res.as_ref().err().unwrap().to_string();
            if err.contains("not have sufficient privilege")
                || err.to_lowercase().contains("access is denied")
            {
                let res = std::os::unix::fs::symlink(&target_symlink, &symlink);

                if res.is_err() {
                    ok = false;
                    println!("{}", err)
                }
            } else if err.contains("file already exists") {
                let res = fs::remove_file(&symlink);
                let mut reloadable = true;
                if reload.len() > 0 {
                    reloadable = reload[0];
                }

                if res.is_err() {
                    println!("{}", res.as_ref().err().unwrap());
                } else if reloadable {
                    let reload = vec![false];
                    self.use_node(&version, &cpu_arch, &reload);
                    return;
                }
            } else {
                println!("{}", err);
            }
        }

        if !ok {
            return;
        }

        // Use the assigned CPu architechture
        let cpu_arch = arch::validate(&cpu_arch);
        let node_path = filepath::join(&self.root, vec![&v_version, "node.exe"]);
        let node32_path = filepath::join(&self.root, vec![&v_version, "node32.exe"]);
        let node64_path = filepath::join(&self.root, vec![&v_version, "node64.exe"]);
        let node_exists = file::exists(&node_path);
        let node32_exists = file::exists(&node32_path);
        let node64_exists = file::exists(&node64_path);

        if node32_exists && cpu_arch == "32" {
            if node_exists {
                let _ = fs::rename(&node_path, &node64_path);
            }
            let _ = fs::rename(&node32_path, &node32_path);
        }
        if node64_exists && cpu_arch == "64" {
            if node_exists {
                let _ = fs::rename(&node_path, &node32_path);
            }
            let _ = fs::rename(node64_path, &node_path);
        }
        println!("Now using node v{} ({}-bit)", version, cpu_arch);
    }

    #[warn(dead_code)]
    fn use_architecture(&mut self, a: &str) {
        let processor_architecture = env::var("PROCESSOR_ARCHITECTURE").unwrap();
        if strings::contains_any("32", &processor_architecture) {
            println!("This computer only supports 32-bit processing.");
            return;
        }
        if a == "32" || a == "64" {
            self.root = a.to_string();
            self.save_settings();
            println!("Set to {}-bit mode", a);
        } else {
            println!(
                "Cannot set architecture to {}. Must be 32 or 64 are accpetable values.",
                a
            );
        }
    }

    fn list(&self, mut list_type: &str) {
        if list_type.is_empty() {
            list_type = "installed";
        }

        if list_type != "installed" && list_type != "available" {
            println!("\nInvalid list option.\n\nPlease use one of the following:\n - nvm list \n - nvm list installed\n - nvm list available");
            help();
            return;
        }

        let v_re = Regex::new("v").unwrap();
        if list_type == "installed" {
            println!();
            let (in_use, arch) = node::get_current_version();
            let installed_versions = node::get_installed(&self.root);

            if installed_versions.is_empty() {
                println!("No installations recognized.");
                return;
            }

            for version in &installed_versions {
                if version.contains("v") {
                    let v_in_use = format!("v{}", in_use);
                    let mut display_version = if v_in_use == *version {
                        format!("* {}", v_re.replace_all(version, ""))
                    } else {
                        format!("  {}", v_re.replace_all(version, ""))
                    };

                    if v_in_use == *version {
                        display_version = format!(
                            "{} (Currently using {}-bit executable)",
                            display_version, arch
                        );
                    }
                    println!("{}\n", display_version);
                }
            }
        } else {
            let (_, lts, current, stable, unstable, _) = node::get_available(&self.ctx.web_ctx);
            let releases = 20;
            let mut data = vec![vec![Version::new(0, 0, 0); 4]; releases];

            for i in 0..releases {
                if let Some(v) = current.get(i) {
                    data[i][0] = v.clone();
                }
                if let Some(v) = lts.get(i) {
                    data[i][1] = v.clone();
                }
                if let Some(v) = stable.get(i) {
                    data[i][2] = v.clone();
                }
                if let Some(v) = unstable.get(i) {
                    data[i][3] = v.clone();
                }
            }

            println!();
            let mut table = Table::new();
            table.max_column_width = 40;

            table.add_row(Row::new(vec![
                TableCell::new_with_alignment("Current", 1, Alignment::Center),
                TableCell::new_with_alignment("LTS", 1, Alignment::Center),
                TableCell::new_with_alignment("Old Stable", 1, Alignment::Center),
                TableCell::new_with_alignment("Old UnStable", 1, Alignment::Center),
            ]));

            for release in data {
                table.add_row(Row::new(vec![
                    TableCell::new_with_alignment(release[0].to_string(), 1, Alignment::Center),
                    TableCell::new_with_alignment(release[1].to_string(), 1, Alignment::Center),
                    TableCell::new_with_alignment(release[2].to_string(), 1, Alignment::Center),
                    TableCell::new_with_alignment(release[3].to_string(), 1, Alignment::Center),
                ]));
            }
            println!("{}", table.render());
        }
    }

    fn enable(&self) {
        let mut dir = String::new();
        let files = fs::read_dir(&self.root).unwrap();

        for t in files {
            let f = t.unwrap();
            let fm = f.metadata().unwrap();

            if fm.is_dir() {
                let filename = f.file_name().into_string().unwrap();
                if filename.contains("v") {
                    dir = filename;
                }
            }
        }
        println!("nvm enabled");
        let trim_c: &[_] = &['\r', '\n'];
        dir = dir.trim_matches(trim_c).to_string();
        if dir.is_empty() {
            println!("No version of node.js found. Try installing the latest by typing nvm install latest");
        } else {
            let re = Regex::new("v").unwrap();
            let version = re.replace_all(&dir, "");
            let reload = vec![];
            self.use_node(version.as_ref(), &self.arch, &reload);
        }
    }

    #[cfg(target_os = "windows")]
    fn disable(&self) {
        let symlink = filepath::clean(&self.symlink);
        match cmd::elevated_run(&self.root, "rmdir", vec![&symlink]) {
            Ok(ok) => {
                if ok {
                    println!("nvm disabled successful");
                } else {
                    println!("nvm disabled fail");
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn disable(&self) {
        let symlink = filepath::clean(&self.symlink);
        match fs::remove_file(&symlink) {
            Ok(_) => {
                println!("nvm disabled successful");
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn disable(&self) {
        let symlink = filepath::clean(&self.symlink);
        match fs::remove_file(&symlink) {
            Ok(_) => {
                println!("nvm disabled successful");
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    #[warn(dead_code)]
    #[cfg(target_os = "windows")]
    fn get_process_permissions() -> WinResult<(bool, bool)> {
        let admin = false;
        let elevated = false;
        Ok((admin, elevated))
    }

    fn setup(&mut self) {
        let lines = file::read_lines(self.settings.as_str());
        if lines.is_err() {
            println!("ERROR:{}", lines.err().unwrap());
            exit(1);
        }
        let lines = lines.unwrap();
        let mut m = HashMap::new();
        for line in lines {
            let line = line.trim();
            strings::replace_env_vars(line);

            let res = line.split(":").collect::<Vec<&str>>();
            if res.len() < 2 {
                continue;
            }
            m.insert(res[0].to_string(), res[1..].join(":").trim().to_string());
        }

        if let Some(root) = m.get("root") {
            self.root = PathBuf::from(root)
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let prefix = r"\\?\";
            if self.root.starts_with(prefix) {
                self.root = self.root[prefix.len()..].to_string();
            }
        }

        if let Some(symlink) = m.get("symlink") {
            self.symlink = symlink.to_string();
        }

        if let Some(original_path) = m.get("originalpath") {
            self.original_path = PathBuf::from(original_path)
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .to_string()
        }

        if let Some(original_version) = m.get("originalversion") {
            self.original_version = original_version.to_string();
        }

        if let Some(arch) = m.get("arch") {
            self.arch = arch.to_string();
        }

        if let Some(node_mirror) = m.get("node_mirror") {
            self.node_mirror = node_mirror.to_string();
        }

        if let Some(npm_mirror) = m.get("npm_mirror") {
            self.npm_mirror = npm_mirror.to_string();
        }

        if let Some(proxy) = m.get("proxy") {
            if proxy != "none" && !proxy.is_empty() {
                if proxy.to_lowercase().chars().take(4).collect::<String>() != "http" {
                    let mut tmp = "http://".to_string();
                    tmp.push_str(proxy.as_str());
                    self.proxy = tmp;
                }
            }
        }

        if let Err(err) = self
            .ctx
            .web_ctx
            .set_proxy(self.proxy.as_str(), self.verify_ssl)
        {
            println!("Set proxy fail,err:{}", err);
        }
        self.arch = arch::validate(&self.arch);

        if !PathBuf::from(&self.root).exists() {
            println!(
                "{} could not be found or does not exist. Exiting.",
                self.root
            );
            exit(1);
        }
    }
}

#[test]
#[cfg(test)]
fn test_environment_new() {
    let env: Environment = Environment::new();
    println!("{:?}", env);
}

#[test]
#[cfg(test)]
fn test_get_version() {
    let env = Environment::new();
    let local_only = vec![false];
    let (v, c) = env.get_version("14.16.0", "64", &local_only).unwrap();
    println!("version:{},cpu:{}", v, c);
}

#[test]
#[cfg(test)]
#[cfg(target_os = "windows")]
fn test_alert() {
    let caption = vec!["测试窗口"];
    Environment::alert("测试消息", caption);
}

#[test]
#[cfg(test)]
#[cfg(target_os = "windows")]
fn test_get_process_permissions() {
    let res = Environment::get_process_permissions();
    println!("{:?}", res);
}

#[test]
#[cfg(test)]
fn test() {
    let mut path: PathBuf = PathBuf::from("/path/to/some/");
    path.push("a");
    path.push("b");

    println!("{}", path.display());
}

#[test]
#[cfg(test)]
fn test_install() {
    let mut env = Environment::new();
    let mut path = env::current_dir().unwrap();
    path.push("tmp");
    env.root = path.to_str().unwrap().to_string();
    env.install("18.20.1", "64");
}
