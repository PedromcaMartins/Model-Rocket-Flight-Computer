use std::{
    io,
    path::PathBuf,
    process::{Child, Command},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

const FC_HOST_STARTUP_TIMEOUT_SECS: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(200);
const GS_RESTART_DELAY: Duration = Duration::from_secs(2);
const MAX_GS_RESTARTS: u32 = 5;

static CTRLC_TRIGGERED: AtomicBool = AtomicBool::new(false);

pub fn run_host() -> Result<()> {
    eprintln!("Building flight-computer-host, ground-station-backend, ground-station-frontend and simulator...");
    build_host_binaries()?;

    ctrlc::set_handler(|| {
        CTRLC_TRIGGERED.store(true, Ordering::Relaxed);
    })
    .context("failed to set Ctrl-C handler")?;

    let target_debug = root_dir().join("target").join("debug");
    let fc_host_bin = target_debug.join("flight-computer-host");
    let gs_backend_bin = target_debug.join("ground-station-backend");
    let gs_frontend_bin = target_debug.join("ground-station-frontend");
    let sim_bin = target_debug.join("host");

    eprintln!("Starting flight-computer-host...");
    let mut fc_host = spawn_in_terminal(
        fc_host_bin.to_str().context("non-UTF8 path")?,
        &[],
        true,
    )
    .context("failed to spawn flight-computer-host")?;

    wait_for_fc_host(&mut fc_host)?;

    eprintln!("Starting ground-station-backend...");
    let gs_bin_str = gs_backend_bin.to_str().context("non-UTF8 path")?.to_owned();
    let mut gs_backend = spawn_in_terminal(&gs_bin_str, &[], false)
        .context("failed to spawn ground-station-backend")?;
    let mut gs_restart_count = 0u32;

    eprintln!("Starting simulator...");
    let mut sim = spawn_in_terminal(
        sim_bin.to_str().context("non-UTF8 path")?,
        &[],
        false,
    )
    .context("failed to spawn simulator")?;

    eprintln!("Starting ground-station-frontend...");
    let gs_fe_bin_str = gs_frontend_bin.to_str().context("non-UTF8 path")?.to_owned();
    let mut gs_frontend = spawn_in_terminal(&gs_fe_bin_str, &[], false)
        .context("failed to spawn ground-station-frontend")?;

    loop {
        if CTRLC_TRIGGERED.load(Ordering::Relaxed) {
            eprintln!("\nShutdown by user");
            kill_child(&mut fc_host);
            kill_child(&mut gs_backend);
            kill_child(&mut gs_frontend);
            kill_child(&mut sim);
            break;
        }

        if let Some(status) = sim.try_wait().context("failed to wait on simulator")? {
            eprintln!("Simulator exited ({status}); terminating FC host, GS backend, and GS frontend");
            kill_child(&mut fc_host);
            kill_child(&mut gs_backend);
            kill_child(&mut gs_frontend);
            break;
        }

        if let Some(status) = fc_host.try_wait().context("failed to wait on FC host")? {
            eprintln!("FC host exited ({status}); terminating simulator, GS backend, and GS frontend");
            kill_child(&mut sim);
            kill_child(&mut gs_backend);
            kill_child(&mut gs_frontend);
            break;
        }

        if let Some(status) = gs_backend.try_wait().context("failed to wait on GS backend")? {
            if status.success() {
                eprintln!("GS backend quit or window closed; shutting down");
                kill_child(&mut fc_host);
                kill_child(&mut sim);
                kill_child(&mut gs_frontend);
                break;
            }
            gs_restart_count += 1;
            if gs_restart_count > MAX_GS_RESTARTS {
                eprintln!("GS backend crashed {gs_restart_count} times; shutting down");
                kill_child(&mut fc_host);
                kill_child(&mut sim);
                kill_child(&mut gs_frontend);
                break;
            }
            eprintln!(
                "GS backend crashed ({status}); restarting ({gs_restart_count}/{MAX_GS_RESTARTS})..."
            );
            std::thread::sleep(GS_RESTART_DELAY);
            gs_backend = spawn_in_terminal(&gs_bin_str, &[], false)
                .context("failed to re-spawn ground-station-backend")?;
            continue;
        }

        // GS frontend lifecycle: observational — its crash leaves the rest running.
        if let Some(status) = gs_frontend.try_wait().context("failed to wait on GS frontend")? {
            if status.success() {
                eprintln!("GS frontend quit or window closed; shutting down");
                kill_child(&mut fc_host);
                kill_child(&mut gs_backend);
                kill_child(&mut sim);
                break;
            }
            eprintln!("GS frontend crashed ({status}); restarting...");
            std::thread::sleep(GS_RESTART_DELAY);
            gs_frontend = spawn_in_terminal(&gs_fe_bin_str, &[], false)
                .context("failed to re-spawn ground-station-frontend")?;
            continue;
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    Ok(())
}

fn wait_for_fc_host(fc_host: &mut Child) -> Result<()> {
    let deadline = Instant::now() + FC_HOST_STARTUP_TIMEOUT_SECS;
    loop {
        if let Some(status) = fc_host.try_wait()? {
            anyhow::bail!("flight-computer-host exited early ({status})");
        }
        if fc_host_socket_ready() {
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!(
                "flight-computer-host did not bind socket within 15s"
            );
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn fc_host_socket_ready() -> bool {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let pipe_path: Vec<u16> = OsStr::new(&format!(r"\\.\pipe\{}", utils::constants::SIM_SOCKET_NAME))
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: WaitNamedPipeW is safe to call with a valid null-terminated wide string.
        unsafe {
            windows_sys::Win32::System::Pipes::WaitNamedPipeW(
                pipe_path.as_ptr(),
                0,
            ) != 0
        }
    }
    #[cfg(not(windows))]
    {
        root_dir().join(utils::constants::SIM_SOCKET_NAME).exists()
    }
}

fn build_host_binaries() -> Result<()> {
    let root = root_dir();
    let sccache = sccache_wrapper();

    if let Some(ref sccache) = sccache {
        eprintln!("Using sccache at: {sccache}");
    } else {
        eprintln!("sccache unavailable; building without cache");
    }

    let builds: [(&str, &[&str]); 4] = [
        ("flight-computer-host", &["build", "-p", "flight-computer-host"]),
        ("ground-station-backend", &["build", "-p", "ground-station-backend"]),
        ("ground-station-frontend", &["build", "-p", "ground-station-frontend"]),
        ("simulator", &["build", "--bin", "host", "-p", "simulator"]),
    ];

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for (name, args) in &builds {
            let root = root.clone();
            let sccache = sccache.clone();
            handles.push(s.spawn(move || -> Result<()> {
                let mut cmd = Command::new("cargo");
                cmd.args(*args);
                cmd.current_dir(&root);
                if let Some(ref sccache) = sccache {
                    cmd.env("RUSTC_WRAPPER", sccache);
                }
                let status = cmd.status()?;
                if status.success() {
                    Ok(())
                } else {
                    anyhow::bail!("{name} build failed ({status})");
                }
            }));
        }

        for h in handles {
            h.join().unwrap()?;
        }

        Ok(())
    })
}

fn sccache_wrapper() -> Option<String> {
    let sccache = "sccache".to_string();
    let sccache = std::env::var_os("SCCACHE_PATH")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| {
            // check the old config location
            let home = std::env::var_os("CARGO_HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .or_else(|| std::env::var_os("HOME"))?;
            let candidate = std::path::Path::new(&home).join(".cargo").join("bin").join("sccache.exe");
            if candidate.is_file() { Some(candidate) } else { None }
        })
        .or_else(|| {
            // fallback: let PATH resolve it
            Some(std::path::PathBuf::from(sccache))
        })?;

    // attempt to start the daemon (no-op if already running)
    let output = std::process::Command::new(&sccache)
        .arg("--start-server")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match output {
        Ok(status) if status.success() => {}
        Ok(_) => {
            eprintln!("warning: sccache daemon failed to start; building without cache");
            return None;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("warning: sccache not found; building without cache");
            return None;
        }
        Err(e) => {
            eprintln!("warning: sccache error ({e}); building without cache");
            return None;
        }
    }

    // verify the daemon is actually responsive
    let ok = std::process::Command::new(&sccache)
        .arg("--show-config")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        Some(sccache.to_string_lossy().into_owned())
    } else {
        eprintln!("warning: sccache daemon unreachable; building without cache");
        None
    }
}

fn spawn_in_terminal(program: &str, args: &[&str], keep_open: bool) -> io::Result<Child> {
    let full_cmd = std::iter::once(program)
        .chain(args.iter().copied())
        .map(|a| {
            if a.contains(' ') || a.contains('"') {
                format!("\"{}\"", a.replace('"', r#"\""#))
            } else {
                a.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let command = if keep_open {
            format!(
                "{}; Write-Host ''; Write-Host 'FC host shut down — press q to close...'; \
                 while(($c = [Console]::ReadKey($true)).KeyChar -ne 'q'){{}}",
                full_cmd
            )
        } else {
            full_cmd
        };
        Command::new("pwsh")
            .arg("-Command")
            .arg(&command)
            .current_dir(root_dir())
            .creation_flags(0x00000010)
            .spawn()
    }
    #[cfg(not(windows))]
    {
        if let Ok(child) = Command::new("x-terminal-emulator")
            .args(["-e", &full_cmd])
            .current_dir(root_dir())
            .spawn()
        {
            return Ok(child);
        }

        Command::new("xterm")
            .args(["-e", &full_cmd])
            .current_dir(root_dir())
            .spawn()
    }
}

fn kill_child(child: &mut Child) {
    #[cfg(windows)]
    {
        let pid = child.id();
        if Command::new("taskkill")
            .args(["/f", "/t", "/pid", &pid.to_string()])
            .status()
            .is_ok_and(|s| s.success())
        {
            let _ = child.wait();
        } else {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    #[cfg(not(windows))]
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
