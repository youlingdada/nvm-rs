

#[test]
#[cfg(test)]
fn test_install(){
    use crate::Environment;

    let mut env: Environment = Environment::new();
    env.setup();
    env.install("18.20.1", "64");
}

#[test]
#[cfg(test)]
fn test_uninstall(){
    use std::env;

    use crate::Environment;

    let mut env = Environment::new();
    let mut path =env::current_dir().unwrap();
    path.push("tmp");
    env.root = path.to_str().unwrap().to_string();
    env.uninstall("18.20.1");
}

#[test]
#[cfg(test)]
fn test_list(){
    use std::env;

    use crate::Environment;

    let mut env = Environment::new();
    let mut path =env::current_dir().unwrap();
    path.push("tmp");
    env.root = path.to_str().unwrap().to_string();
    env.list("available");   
}

#[test]
#[cfg(test)]
fn test_setup(){
    use std::env;

    match env::current_exe() {
        Ok(exe_path) => {
            println!("Current executable path: {:?}", exe_path);
        }
        Err(e) => {
            println!("Failed to get current executable path: {}", e);
        }
    }
}

#[test]
#[cfg(test)]
fn test(){
    // 获取当前cpu 架构

    let architecture = std::env::consts::ARCH;
    println!("CPU Architecture: {}", architecture);
}
