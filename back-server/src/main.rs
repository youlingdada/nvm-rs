use std::fs;
use std::path::Path;
use tokio::net::UdpSocket;

async fn handle_request(
    socket: &UdpSocket,
    buf: &[u8],
    src: std::net::SocketAddr,
) -> tokio::io::Result<()> {
    let request = String::from_utf8_lossy(buf).trim().to_string();
    let parts: Vec<&str> = request.split(' ').collect();
    let command = parts[0];
    let path = parts.get(1).unwrap_or(&"").to_string();

    let response = match command {
        "DELETE_DIR" => {
            if Path::new(&path).is_dir() {
                fs::remove_dir_all(&path)?;
                "DELETE_DIR_YES".to_string()
            } else {
                "DELETE_DIR_NO".to_string()
            }
        }
        "CREATE_DIR" => {
            if !Path::new(&path).exists() {
                fs::create_dir_all(&path)?;
                "CREATE_DIR_YES".to_string()
            } else {
                "CREATE_DIR_EXISTS".to_string()
            }
        }
        "CREATE_SYMLINK_DIR" => {
            if let Some(target) = parts.get(2) {
                if !Path::new(&path).exists() {
                    #[cfg(target_os = "windows")]
                    std::os::windows::fs::symlink_dir(target, &path)?;
                    #[cfg(target_os = "linux")]
                    std::os::unix::fs::symlink(target, &path)?;
                    #[cfg(target_os = "macos")]
                    std::os::unix::fs::symlink(target, &path)?;
                    "CREATE_SYMLINK_DIR_YES".to_string()
                } else {
                    "CREATE_SYMLINK_DIR_EXISTS".to_string()
                }
            } else {
                "CREATE_SYMLINK_DIR_NO".to_string()
            }
        }
        "CREATE_SYMLINK_FILE" => {
            if let Some(target) = parts.get(2) {
                if !Path::new(&path).exists() {
                    #[cfg(target_os = "windows")]
                    std::os::windows::fs::symlink_file(target, &path)?;
                    #[cfg(target_os = "linux")]
                    std::os::unix::fs::symlink(target, &path)?;
                    #[cfg(target_os = "macos")]
                    std::os::unix::fs::symlink(target, &path)?;
                    "CREATE_SYMLINK_FILE_YES".to_string()
                } else {
                    "CREATE_SYMLINK_FILE_EXISTS".to_string()
                }
            } else {
                "CREATE_SYMLINK_FILE_NO".to_string()
            }
        }
        "DELETE_FILE" => {
            if Path::new(&path).is_file() {
                fs::remove_file(&path)?;
                "DELETE_FILE_YES".to_string()
            } else {
                "DELETE_FILE_NO".to_string()
            }
        }
        _ => "Unknown command".to_string(),
    };

    socket.send_to(response.as_bytes(), src).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:7878").await?;
    println!("Daemon is running...");

    let mut buf = [0; 1024];
    loop {
        let (n, src) = socket.recv_from(&mut buf).await?;
        if let Err(e) = handle_request(&socket, &buf[..n], src).await {
            eprintln!("Error handling request: {}", e);
            socket.send_to(e.to_string().as_bytes(), src).await?;
        }
    }
}
