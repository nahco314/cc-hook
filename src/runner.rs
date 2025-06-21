use crate::{config::Config, frame::FrameDetector, hook::HookEngine, screen::ScreenManager};
use nix::{
    pty::{openpty, OpenptyResult},
    sys::{
        termios::{self, SetArg},
        wait::{waitpid, WaitStatus},
    },
    unistd::{close, dup2, execvp, fork, getpid, isatty, setsid, tcsetpgrp, ForkResult},
};
use std::{
    ffi::CString,
    os::unix::io::{AsRawFd, BorrowedFd, FromRawFd},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
    signal::unix::{signal, SignalKind},
    sync::mpsc,
    time::sleep,
};

pub async fn run_with_hooks(
    args: Vec<String>,
    config: Config,
) -> Result<i32, Box<dyn std::error::Error>> {
    let OpenptyResult { master, slave } = openpty(None, None)?;
    let master_fd = master.as_raw_fd();
    let slave_fd = slave.as_raw_fd();

    // Save original terminal settings if we're in a TTY
    let stdin = std::io::stdin();
    let is_tty = isatty(stdin.as_raw_fd()).unwrap_or(false);
    let orig_termios = if is_tty {
        Some(termios::tcgetattr(&stdin)?)
    } else {
        None
    };

    let child_pid = match unsafe { fork() }? {
        ForkResult::Parent { child } => child,
        ForkResult::Child => {
            close(master_fd)?;

            // Create new session and become session leader
            setsid()?;

            dup2(slave_fd, 0)?;
            dup2(slave_fd, 1)?;
            dup2(slave_fd, 2)?;
            close(slave_fd)?;

            // Make this process the foreground process group if we're in a TTY
            if is_tty {
                let slave_stdin = std::io::stdin();
                let _ = tcsetpgrp(&slave_stdin, getpid());
            }

            let cmd = CString::new(args[0].as_bytes())?;
            let args: Vec<CString> = args
                .into_iter()
                .map(|s| CString::new(s.as_bytes()).unwrap())
                .collect();

            execvp(&cmd, &args)?;
            unreachable!();
        }
    };

    drop(slave);

    // Put terminal in raw mode if we're in a TTY
    if let Some(ref orig) = orig_termios {
        let mut raw_termios = orig.clone();
        termios::cfmakeraw(&mut raw_termios);
        termios::tcsetattr(&stdin, SetArg::TCSANOW, &raw_termios)?;

        // Copy terminal settings to PTY
        let master_borrowed = unsafe { BorrowedFd::borrow_raw(master_fd) };
        termios::tcsetattr(master_borrowed, SetArg::TCSANOW, orig)?;
    }

    let mut screen = ScreenManager::new(24, 80);
    let mut frame_detector = FrameDetector::new();
    let mut hook_engine = HookEngine::new(config.hooks)?;

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);

    let master_dup = nix::unistd::dup(master_fd)?;
    let master_async = tokio::fs::File::from_std(unsafe { std::fs::File::from_raw_fd(master_dup) });

    let mut reader = tokio::io::BufReader::new(master_async.try_clone().await?);
    let writer = master_async;

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut buf = vec![0; 4096];
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if tx_clone.send(buf[..n].to_vec()).await.is_err() {
                        break;
                    }
                }
                Err(_) => {
                    // Child process closed PTY
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut buf = vec![0; 4096];
        let mut stdin = stdin;
        let mut writer = writer;
        loop {
            match stdin.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if writer.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => {
                    // Stdin closed or error
                    break;
                }
            }
        }
    });

    let mut sigwinch = signal(SignalKind::window_change())?;

    // Ensure we restore terminal settings on exit
    let stdin_for_cleanup = std::io::stdin();
    let orig_termios_for_cleanup = orig_termios.clone();
    let cleanup = || {
        if let Some(ref orig) = orig_termios_for_cleanup {
            let _ = termios::tcsetattr(&stdin_for_cleanup, SetArg::TCSANOW, orig);
        }
    };

    let result = async {
        loop {
            select! {
                Some(data) = rx.recv() => {
                    stdout.write_all(&data).await?;
                    stdout.flush().await?;

                    screen.process(&data);
                    frame_detector.on_data(&data);
                }
                _ = sigwinch.recv() => {
                    let stdin = std::io::stdin();
                    if let Ok(size) = termios::tcgetattr(&stdin) {
                        let master_borrowed = unsafe { BorrowedFd::borrow_raw(master_fd) };
                        termios::tcsetattr(master_borrowed, SetArg::TCSANOW, &size)?;
                    }
                }
                _ = sleep(std::time::Duration::from_millis(10)) => {
                    // Check for frame boundary
                    if frame_detector.should_capture_frame() {
                        let (prev, curr) = screen.take_snapshot();
                        let commands = hook_engine.evaluate(&prev, &curr);
                        HookEngine::execute_commands(commands);
                        frame_detector.reset();
                    }

                    // Check if child process has exited
                    match waitpid(child_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG))? {
                        WaitStatus::Exited(_, code) => return Ok(code),
                        WaitStatus::Signaled(_, sig, _) => return Ok(128 + sig as i32),
                        _ => continue,
                    }
                }
            }
        }
    }
    .await;

    // Restore terminal settings
    cleanup();

    result
}
