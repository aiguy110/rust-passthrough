use std::fs::{self, OpenOptions};
use std::process::{Command, Stdio, exit};
use std::io::{self, Write, Read};

fn main() {
    // Read the path of the executable from /etc/rust_passthrough.cmd
    let cmd_path = match fs::read_to_string("/etc/rust_passthrough.cmd") {
        Ok(path) => path.trim().to_string(),
        Err(err) => {
            eprintln!("Error reading /etc/rust_passthrough.cmd: {}", err);
            exit(1);
        }
    };

    // Gather arguments passed to the rust_passthrough program
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Open the output and error files for writing
    let mut out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("/tmp/rust_passthrough.out")
        .unwrap();

    let mut err_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("/tmp/rust_passthrough.err")
        .unwrap();

    // Execute the command as a subprocess
    let mut child = Command::new(&cmd_path)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("Failed to execute command {}: {}", cmd_path, err);
            exit(1);
        });

    // Capture and handle stdout and stderr of the subprocess
    if let Some(mut out_pipe) = child.stdout.take() {
        let mut buffer = Vec::new();
        out_pipe.read_to_end(&mut buffer).unwrap();

        io::stdout().write_all(&buffer).unwrap();
        out_file.write_all(&buffer).unwrap();
    }

    if let Some(mut err_pipe) = child.stderr.take() {
        let mut buffer = Vec::new();
        err_pipe.read_to_end(&mut buffer).unwrap();

        io::stderr().write_all(&buffer).unwrap();
        err_file.write_all(&buffer).unwrap();
    }

    let result = child.wait().unwrap_or_else(|err| {
        eprintln!("Failed to wait for {}: {}", cmd_path, err);
        exit(1);
    });

    if !result.success() {
        eprintln!("Subprocess did not complete successfully");
        exit(result.code().unwrap_or(1));
    }
}

