// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Install service: privileged operations for installing the target system

use crate::auth::AuthService;
use disks::BlockDevice;
use lichen_macros::authorized;
use protocols::lichen::install::{
    DiscoverSystemModelsResponse, DiscoveredModel, InstallProgress, InstallSystemRequest, WriteSystemModelRequest,
    WriteSystemModelResponse,
    install_server::{Install, InstallServer},
};
use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Where the target root is temp mounted while writing
const TARGET_MOUNT: &str = "/run/lichen/target";
// Where candidate partitions are briefly mounted read-only while probing
/// for a previous installation's system model
const PROBE_MOUNT: &str = "/run/lichen/probe";
/// Location of the system model inside the target root; moss reads and
/// rewrites this exact path on the installed system
const MODEL_PATH: &str = "usr/lib/system-model.kdl";
/// The installer's permanent record on the target: the install-model superset
/// wrapping the sytem-model
const INSTALL_MODEL_PATH: &str = "etc/moss/install-model.kdl";

/// Service represents the install service implementation
#[derive(Debug)]
pub struct Service {
    auth: Arc<AuthService>,
}

/// Creates a new Install gRPC server instance using the default Service implementation
pub fn service(auth: Arc<AuthService>) -> InstallServer<Service> {
    InstallServer::new(Service { auth })
}

#[tonic::async_trait]
impl Install for Service {
    type InstallSystemStream = ReceiverStream<Result<InstallProgress, Status>>;

    #[authorized("com.aerynos.lichen.install.write-model")]
    async fn write_system_model(
        &self,
        request: Request<WriteSystemModelRequest>,
    ) -> Result<Response<WriteSystemModelResponse>, tonic::Status> {
        let req = request.into_inner();
        info!(root_device = %req.root_device, "Writing system model target");

        if req.root_device.is_empty() {
            return Err(Status::invalid_argument("no root device provided"));
        }
        if !Path::new(&req.root_device).exists() {
            return Err(Status::not_found(format!("no such device: {}", req.root_device)));
        }

        tokio::task::block_in_place(|| write_to_target(&req.root_device, &req.contents, &req.install_model))?;

        Ok(Response::new(WriteSystemModelResponse {}))
    }

    #[authorized("com.aerynos.lichen.install.discover")]
    async fn discover_system_models(
        &self,
        request: Request<()>,
    ) -> Result<Response<DiscoverSystemModelsResponse>, tonic::Status> {
        let _ = request;
        info!("Probing disks for previous installation system models");
        let models = tokio::task::block_in_place(discover_models)?;

        Ok(Response::new(DiscoverSystemModelsResponse { models }))
    }

    #[authorized("com.aerynos.lichen.install.system")]
    async fn install_system(
        &self,
        request: Request<InstallSystemRequest>,
    ) -> Result<Response<Self::InstallSystemStream>, tonic::Status> {
        let req = request.into_inner();
        if !req.mounts.iter().any(|mount| mount.mountpoint == "/") {
            return Err(Status::invalid_argument("no root mount provided"));
        }

        info!("Installing system to target");

        let (tx, rx) = mpsc::channel(64);
        let done = Arc::new(AtomicBool::new(false));

        // Keep-alive ticks so the stream never idles, even while moss is quiet
        {
            let tx = tx.clone();
            let done = done.clone();

            thread::spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_secs(10));

                    let update = InstallProgress {
                        message: String::new(),
                        finished: false,
                    };

                    if tx.blocking_send(Ok(update)).is_err() {
                        break;
                    }
                }
            });
        }

        thread::spawn(move || {
            let progress = |message: String| {
                let _ = tx.blocking_send(Ok(InstallProgress {
                    message,
                    finished: false,
                }));
            };
            let result = install_target(&req, &progress);

            done.store(true, Ordering::Relaxed);
            match result {
                Ok(()) => {
                    let _ = tx.blocking_send(Ok(InstallProgress {
                        message: "Installation complete".to_string(),
                        finished: true,
                    }));
                }
                Err(status) => {
                    let _ = tx.blocking_send(Err(status));
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// Mount the target root, write the model, and always unmount again,
/// even when the write fails
fn write_to_target(root_device: &str, contents: &str, install_model: &str) -> Result<(), Status> {
    let target = Path::new(TARGET_MOUNT);

    fs::create_dir_all(target)?;

    run(Command::new("mount").args([root_device, target.to_str().expect("expected a target")]))?;

    let result = (|| -> Result<(), Status> {
        let model_path = target.join(MODEL_PATH);
        let install_model_path = target.join(INSTALL_MODEL_PATH);

        if let Some(parent) = model_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Some(parent) = install_model_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&model_path, contents)?;
        fs::write(&install_model_path, install_model)?;

        Ok(())
    })();

    let unmounted = run(Command::new("umount").arg(target));

    result.and(unmounted)
}

/// Probe every unmounted partition read-only for a system model from a previous
/// installation. Unmountable partitions are auto skipped.
fn discover_models() -> Result<Vec<DiscoveredModel>, Status> {
    let probe = Path::new(PROBE_MOUNT);
    fs::create_dir_all(probe)?;

    let mounted = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    let devices = BlockDevice::discover()?;
    let mut models = Vec::new();

    for device in &devices {
        for part in device.partitions() {
            let node = part.device.display().to_string();

            // Never touch partitions that are already mounted somewhere
            if mounted.lines().any(|line| line.starts_with(&format!("{node} "))) {
                continue;
            }

            // No mountable filesystem means not a candidate
            if run(Command::new("mount").args(["-o", "ro", &node, &probe.to_string_lossy()])).is_err() {
                continue;
            }

            let contents = fs::read_to_string(probe.join(INSTALL_MODEL_PATH))
                .or_else(|_| fs::read_to_string(probe.join(MODEL_PATH)))
                .ok();
            let _ = run(Command::new("umount").arg(probe));

            if let Some(contents) = contents {
                info!(device = %node, "Found system model from a previous installation");
                models.push(DiscoveredModel { device: node, contents });
            }
        }
    }

    Ok(models)
}

/// Mount the target filesystems, install the OS via moss from the system
/// model written earlier, configure the target, and always unmount again
fn install_target(req: &InstallSystemRequest, progress: &(dyn Fn(String) + Sync)) -> Result<(), Status> {
    let target = Path::new(TARGET_MOUNT);
    fs::create_dir_all(target)?;

    // Root first, nested boot mounts after
    let mut mounts = req.mounts.clone();
    mounts.sort_by_key(|mount| mount.mountpoint.len());

    let mut mounted: Vec<PathBuf> = Vec::new();
    let result = (|| -> Result<(), Status> {
        progress("Mounting target filesystems".to_string());
        for mount in &mounts {
            let mountpoint = target.join(mount.mountpoint.trim_start_matches('/'));
            fs::create_dir_all(&mountpoint)?;
            run(Command::new("mount").args([&mount.device, &mountpoint.to_string_lossy().into()]))?;
            mounted.push(mountpoint);
        }

        // Virtual filesystems needed by moss triggers and chroot commands
        for (source, dest) in [
            ("/dev", "dev"),
            ("/dev/shm", "dev/shm"),
            ("/dev/pts", "dev/pts"),
            ("/proc", "proc"),
            ("/sys", "sys"),
        ] {
            let mountpoint = target.join(dest);
            fs::create_dir_all(&mountpoint)?;
            run(Command::new("mount").args(["--bind", source, &mountpoint.to_string_lossy()]))?;
            mounted.push(mountpoint);
        }

        // `moss sync --import` does not bootstrap repos on an empty root
        if req.repositories.is_empty() {
            warn!("no repos to prime; sync will fail unless moss bootstraps them itself");
        }

        for repo in &req.repositories {
            progress(format!("Priming repository {}", repo.id));
            info!(repo = %repo.id, uri = %repo.uri, "Priming repository in target");
            run(Command::new("moss")
                .arg("-D")
                .arg(target)
                .args(["repo", "add"])
                .arg(&repo.id)
                .arg(&repo.uri))?;
        }

        // moss materializes the system from the model, including populating
        // the mounted ESP/XBOOTLDR with boot entries via its blsforme
        // integration, which is why the boot mounts must be live first
        progress("Installing packages".to_string());
        info!("Running moss sync agains the target (this can take a while)");
        run_streaming(
            Command::new("moss")
                .args(["sync", "--import"])
                .arg(target.join(MODEL_PATH))
                .arg("-D")
                .arg(target)
                .arg("-y"),
            progress,
        )?;

        progress("Configuring target system".to_string());
        configure_target(target, req)
    })();

    // Unwind every mount in reverse order regardless of the outcome
    progress("Unmounting target filesystems".to_string());
    for mountpoint in mounted.iter().rev() {
        let _ = run(Command::new("umount").arg(mountpoint));
    }
    let _ = run(&mut Command::new("sync"));

    result
}

/// Apply the installer-owned config to the installed target
fn configure_target(target: &Path, req: &InstallSystemRequest) -> Result<(), Status> {
    if !req.locale.is_empty() {
        fs::write(target.join("etc/locale.conf"), format!("LANG={}\n", req.locale))?;
    }

    if !req.timezone.is_empty() {
        let localtime = target.join("etc/localtime");
        let _ = fs::remove_file(&localtime);
        unix::fs::symlink(format!("../usr/share/zoneinfo/{}", req.timezone), &localtime)?;
    }

    let _ = fs::remove_file(target.join("etc/machine-id"));
    run(Command::new("chroot").arg(target).arg("systemd-machine-id-setup"))?;

    if let Some(user) = &req.user {
        run(Command::new("chroot")
            .arg(target)
            .args([
                "useradd",
                "-m",
                "-U",
                "-G",
                "audio,adm,wheel,render,kvm,input,users",
                "-c",
            ])
            .args([&user.real_name, &user.username]))?;
    }

    let mut entries = String::new();
    if !req.root_password_hash.is_empty() {
        entries.push_str(&format!("root:{}\n", req.root_password_hash));
    }
    if let Some(user) = &req.user {
        entries.push_str(&format!("{}:{}\n", user.username, user.password_hash));
    }
    if !entries.is_empty() {
        set_passwords(target, &entries)?;
    }

    // fstab root entry
    if let Some(root) = req.mounts.iter().find(|mount| mount.mountpoint == "/") {
        let partuuid = blkid(&root.device, "PARTUUID")?;
        let fstype = blkid(&root.device, "TYPE")?;
        fs::write(
            target.join("etc//fstab"),
            format!("# /etc/fstab: static filesystem information.\nPARTUUID={partuuid} / {fstype} defaults 0 1\n"),
        )?;
    }

    Ok(())
}

/// Set account passwords from pre-computed crypt(3) hashes via chpasswd -e
fn set_passwords(target: &Path, entries: &str) -> Result<(), Status> {
    let mut child = Command::new("chroot")
        .arg(target)
        .args(["chpasswd", "-e"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Status::internal(format!("failed to spawn chpasswd: {e}")))?;

    child
        .stdin
        .take()
        .expect("stdin was piped")
        .write_all(entries.as_bytes())?;

    let output = child
        .wait_with_output()
        .map_err(|e| Status::internal(format!("chpasswd did not complete: {e}")))?;

    if !output.status.success() {
        return Err(Status::internal(format!(
            "chpasswd failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Read a single blkid tag value from a device
fn blkid(device: &str, tag: &str) -> Result<String, Status> {
    let output = Command::new("blkid")
        .args(["-s", tag, "-o", "value"])
        .arg(device)
        .output()
        .map_err(|e| Status::internal(format!("failed to spawn blkid: {e}")))?;

    if !output.status.success() {
        return Err(Status::internal(format!("blkid {tag} failed for {device}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run a long command forwarding its output lines as progress, keeping
/// tail of recent lines for error reporting
fn run_streaming(command: &mut Command, progress: &(dyn Fn(String) + Sync)) -> Result<(), Status> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Status::internal(format!("failed to spawn {:?}: {e}", command.get_program())))?;
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let tail: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let record = |line: &str| {
        let mut tail = tail.lock().unwrap();
        if tail.len() >= 30 {
            tail.pop_front();
        }
        tail.push_back(line.to_string());
    };

    thread::scope(|scope| {
        scope.spawn(|| {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                record(&line);
                warn!("stderr: {line}");
            }
        });

        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            record(&line);

            let cleaned = clean_line(&line);
            if !cleaned.is_empty() {
                progress(cleaned);
            }
        }
    });

    let status = child
        .wait()
        .map_err(|e| Status::internal(format!("{:?} did not complete: {e}", command.get_program())))?;

    if !status.success() {
        let detail = tail.lock().unwrap().iter().cloned().collect::<Vec<_>>().join("\n");
        return Err(Status::internal(format!(
            "{:?} failed ({}): {}",
            command.get_program(),
            status,
            detail,
        )));
    }

    Ok(())
}

/// Reduce a raw output line to something fit for a one-line progress display
fn clean_line(line: &str) -> String {
    let last = line.rsplit('\r').next().unwrap_or(line);
    let mut out = String::with_capacity(last.len());
    let mut chars = last.chars();

    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.next() == Some('[') {
                for end in chars.by_ref() {
                    if end.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        if !c.is_control() {
            out.push(c);
        }
    }
    out.trim().to_string()
}

/// Run a command to completion, mapping failure to a gRPC status carrying
/// the command's stderr
fn run(command: &mut Command) -> Result<(), Status> {
    let output = command
        .output()
        .map_err(|e| Status::internal(format!("failed to spawn {:?}: {e}", command.get_program())))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() { stdout } else { stderr };

        return Err(Status::internal(format!(
            "{:?} failed ({}): {}",
            command.get_program(),
            output.status,
            detail.trim()
        )));
    }
    Ok(())
}
