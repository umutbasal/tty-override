use nix::errno::Errno;
use nix::pty::{ForkptyResult, Winsize};
use nix::sys::wait;
use nix::unistd::{self, ForkResult, Pid};
use std::env;
use std::ffi::CString;
use std::io::{self, Result, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::process;

mod config;
use config::Config;

// Constants for standard file descriptors
const STDIN: BorrowedFd = unsafe { BorrowedFd::borrow_raw(0) };
const STDOUT: BorrowedFd = unsafe { BorrowedFd::borrow_raw(1) };
const STDERR: BorrowedFd = unsafe { BorrowedFd::borrow_raw(2) };

fn main() -> Result<()> {
    // Load configuration from file
    let cfg = load_config()?;

    // Clone standard input and error file descriptors
    let stdin = STDIN.try_clone_to_owned()?;
    let stderr = STDERR.try_clone_to_owned()?;

    // Fork a process for standard output redirection
    let forkout = unsafe { fork_pty()? };
    if let ForkResult::Parent { child } = forkout.fork_result {
        redirect_output(forkout.master.as_fd(), STDOUT, cfg);
        process::exit(wait::waitpid(child, None).is_ok() as i32);
    }

    // Clone standard output file descriptor
    let stdout = STDOUT.try_clone_to_owned()?;

    // Fork a process for standard error redirection
    let forkerr = unsafe { fork_pty()? };
    if let ForkResult::Parent { child } = forkerr.fork_result {
        redirect_output(forkerr.master.as_fd(), stderr.as_fd(), cfg);
        process::exit(wait::waitpid(child, None).is_ok() as i32);
    }

    // Redirect standard input and output back to the original ones for the child process
    unistd::dup2(stdin.as_raw_fd(), STDIN.as_raw_fd())?;
    unistd::dup2(stdout.as_raw_fd(), STDOUT.as_raw_fd())?;

    // Execute the command
    match execute_command(env::args().skip(1).collect()) {
        Ok(_) => {}
        Err(e) => {
            let _ = writeln!(io::stderr(), "tty-override: {}\n", e);
            process::exit(1);
        }
    }
    process::exit(0);
}

// Load configuration from file
fn load_config() -> Result<Config> {
    let home = std::env::var("HOME").expect("HOME not set");
    let path = format!("{}/.config/tty-override/config.toml", home);
    Config::from_file("./config/config.toml")
        .or(Config::from_file(&path))
        .map_err(|e| e)
}

// Execute the command with given arguments
fn execute_command(args: Vec<String>) -> Result<()> {
    let args: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.as_bytes()).unwrap())
        .collect();
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    if args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no command"));
    }
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

// Redirect output based on configuration rules
fn redirect_output(r: BorrowedFd, w: BorrowedFd, cfg: Config) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(r.as_raw_fd(), &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let mut output = String::from_utf8_lossy(&buf[..n]).to_string();
                // TODO: Implement tty-override program <arg> matching
                //if profile.program == "gh-copilot" && profile.argmatch == "*" {
                for profile in &cfg.profiles {
                    for rule in &profile.rules {
                        output = rule
                            .pattern
                            .replace_all(&output, &rule.replacement)
                            .to_string();
                    }
                }
                let lines: Vec<String> = output.lines().map(|line| line.to_string()).collect();
                let prefixed_output = lines.join("\n");
                let mut prefb = prefixed_output.as_bytes();
                while !prefb.is_empty() {
                    let written = unistd::write(w, prefb).unwrap();
                    prefb = &prefb[written..];
                }
            }
        }
    }
}

// Fork a new process with a pseudo-terminal
unsafe fn fork_pty() -> Result<ForkptyResult> {
    let mut winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut master = std::mem::MaybeUninit::<libc::c_int>::uninit();
    let res = libc::forkpty(
        master.as_mut_ptr(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut winsize,
    );
    let fork_result = Errno::result(res).map(|res| match res {
        0 => ForkResult::Child,
        res => ForkResult::Parent {
            child: Pid::from_raw(res),
        },
    })?;
    Ok(ForkptyResult {
        master: OwnedFd::from_raw_fd(master.assume_init()),
        fork_result,
    })
}
