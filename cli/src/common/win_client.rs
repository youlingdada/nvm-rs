use tokio::{net::UdpSocket, runtime::Runtime};

async fn send_request(
    command: &str,
    path: &str,
    target: Option<&str>,
) -> tokio::io::Result<String> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;
    let request = if let Some(t) = target {
        format!("{} {} {}", command, path, t)
    } else {
        format!("{} {}", command, path)
    };

    socket.send_to(request.as_bytes(), "127.0.0.1:7878").await?;

    let mut buf = [0; 1024];
    let (n, _) = socket.recv_from(&mut buf).await?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

fn send_request_sync(
    runtime: &Runtime,
    command: &str,
    path: &str,
    target: Option<&str>,
) -> tokio::io::Result<String> {
    runtime.block_on(async { send_request(command, path, target).await })
}

pub fn delete_dir(runtime: &Runtime, path: &str) -> Result<bool, String> {
    match send_request_sync(runtime, "DELETE_DIR", path, None) {
        Ok(response) => {
            if response == "DELETE_DIR_YES" {
                Ok(true)
            } else {
                Err(response)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_dir(runtime: &Runtime, path: &str) -> Result<bool, String> {
    match send_request_sync(runtime, "CREATE_DIR", path, None) {
        Ok(response) => {
            if response == "CREATE_DIR_YES" {
                Ok(true)
            } else {
                Err(response)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn create_symlink(runtime: &Runtime, path: &str, target: &str) -> Result<bool, String> {
    match send_request_sync(runtime, "CREATE_SYMLINK", path, Some(target)) {
        Ok(response) => {
            if response == "CREATE_SYMLINK_YES" {
                Ok(true)
            } else {
                Err(response)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_file(runtime: &Runtime, path: &str) -> Result<bool, String> {
    match send_request_sync(runtime, "DELETE_FILE", path, None) {
        Ok(response) => {
            if response == "DELETE_FILE_YES" {
                Ok(true)
            } else {
                Err(response)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[test]
#[cfg(test)]
fn create_dir_test() {
    let runtime = Runtime::new().unwrap();
    println!("{:?}", create_dir(&runtime, "C:\\tmp\\test"));
}

#[test]
#[cfg(test)]
fn delete_dir_test() {
    let runtime = Runtime::new().unwrap();
    println!("{:?}", delete_dir(&runtime, "C:\\tmp\\test"));
}

#[test]
#[cfg(test)]
fn create_symlink_test() {
    let runtime = Runtime::new().unwrap();
    println!("{:?}", create_symlink(&runtime, "C:\\tmp\\test", "C:\\tmp\\test2"));
}

#[test]
#[cfg(test)]
fn delete_file_test() {
    let runtime = Runtime::new().unwrap();
    println!("{:?}", delete_file(&runtime, "C:\\tmp\\test2"));
}
