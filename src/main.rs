use nix::errno::Errno;
use nix::pty::{ForkptyResult, Winsize};
use nix::sys::wait::{self};
use nix::unistd::{self, ForkResult, Pid};
use std::env;
use std::ffi::CString;
use std::io::{self, Result, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::process;

mod config;
use config::Config;

fn main() -> Result<()> {
    let home = std::env::var("HOME").expect("HOME not set");
    let path = &format!("{}/tty-override/config/config.toml", home);
    let cfg: Config = Config::from_file("./config/config.toml")
        .or(Config::from_file(path))
        .expect("failed to read");

    let stdin = STDIN.try_clone_to_owned()?;
    let stderr = STDERR.try_clone_to_owned()?;
    let forkout = unsafe { forkpty() }?;
    if let ForkResult::Parent { child } = forkout.fork_result {
        cpoverride(forkout.master.as_fd(), STDOUT, cfg);
        process::exit(wait::waitpid(child, None).is_ok() as i32);
    }
    let stdout = STDOUT.try_clone_to_owned()?;
    let forkerr = unsafe { forkpty() }?;
    if let ForkResult::Parent { child } = forkerr.fork_result {
        cpoverride(forkerr.master.as_fd(), stderr.as_fd(), cfg);
        process::exit(wait::waitpid(child, None).is_ok() as i32);
    }
    unistd::dup2(stdin.as_raw_fd(), STDIN.as_raw_fd())?;
    unistd::dup2(stdout.as_raw_fd(), STDOUT.as_raw_fd())?;

    let args: Vec<CString> = env::args()
        .skip(1)
        .map(|os_string| CString::new(os_string.as_bytes()).unwrap())
        .collect();
    match exec(args) {
        Ok(_) => {}
        Err(e) => {
            let _ = writeln!(io::stderr(), "tty-override: {}\n", e);
            process::exit(1);
        }
    }
    process::exit(0);
}

fn exec(args: Vec<CString>) -> Result<()> {
    let args: Vec<_> = args.iter().map(CString::as_c_str).collect();
    if args.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no command"));
    }
    unistd::execvp(args[0], &args)?;
    unreachable!();
}

fn cpoverride(r: BorrowedFd, w: BorrowedFd, cfg: Config) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(r.as_raw_fd(), &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let mut output = String::from_utf8_lossy(&buf[..n]).to_string();

                for profile in &cfg.profiles {
                    // TODO: Implement tty-override program <arg> matching
                    //if profile.program == "gh-copilot" && profile.argmatch == "*" {
                    for rule in &profile.rules {
                        output = rule
                            .pattern
                            .replace_all(&output, &rule.replacement)
                            .to_string();
                    }
                    //}
                }

                let lines: Vec<String> = output.lines().map(|line| line.to_string()).collect();

                let prefixed_output = lines.join("\n");
                let mut prefb = prefixed_output.as_bytes();

                while !prefb.is_empty() {
                    let n = unistd::write(w, prefb).unwrap();
                    prefb = &prefb[n..];
                }
            }
        }
    }
}

const STDIN: BorrowedFd = unsafe { BorrowedFd::borrow_raw(0) };
const STDOUT: BorrowedFd = unsafe { BorrowedFd::borrow_raw(1) };
const STDERR: BorrowedFd = unsafe { BorrowedFd::borrow_raw(2) };

unsafe fn forkpty() -> Result<ForkptyResult> {
    let mut winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    use std::mem;
    use std::ptr;

    let mut master = mem::MaybeUninit::<libc::c_int>::uninit();

    let res = unsafe {
        libc::forkpty(
            master.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut winsize,
        )
    };

    let fork_result = Errno::result(res).map(|res| match res {
        0 => ForkResult::Child,
        res => ForkResult::Parent {
            child: Pid::from_raw(res),
        },
    })?;

    Ok(ForkptyResult {
        master: unsafe { OwnedFd::from_raw_fd(master.assume_init()) },
        fork_result,
    })
}
