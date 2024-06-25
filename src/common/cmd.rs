use std::process::Command;

pub fn elevated_run(root: &str, name: &str, args: Vec<&str>) -> Result<bool, String> {
    let mut cmd_arg = args.clone();
    cmd_arg.insert(0, "/C");
    cmd_arg.insert(1, name);

    match run("cmd", None, &cmd_arg) {
        Ok(_) => Ok(true),
        Err(_) => {
            cmd_arg.insert(0, "cmd");
            run("elevate.cmd", Some(root), &cmd_arg)
        }
    }
}

fn run(name: &str, dir: Option<&str>, args: &Vec<&str>) -> Result<bool, String> {
    let mut cmd = Command::new(name);

    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    cmd.args(args);

    // print_command(&cmd);

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Command failed with error: {}", stderr))
            } else {
                Ok(true)
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}
