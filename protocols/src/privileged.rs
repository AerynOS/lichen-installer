// SPDX-FileCopyrightText: Copyright © 2020-2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Provides facilities for privilege escalation and service management using Unix domain sockets.
//!
//! This module enables creating privileged services that can be accessed through Unix domain sockets,
//! with support for both direct execution and privilege escalation via pkexec.

use std::{
    env,
    io::{self},
    os::{
        fd::{FromRawFd, OwnedFd, RawFd},
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener, UnixStream},
    },
    process::Command,
};

use command_fds::{CommandFdExt, FdMapping, FdMappingCollision};
use nix::unistd::Pid;
use std::ops::Deref;
use thiserror::Error;

/// Errors that can occur when working with privileged services
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred while spawning the privileged worker
    #[error("Failed to spawn privileged worker: {0}")]
    IO(#[from] io::Error),

    /// A file descriptor mapping collision occurred
    #[error("mapping collision@ {0}")]
    MappingCollision(#[from] FdMappingCollision),

    /// The fork operation failed
    #[error("Failed to fork: {0}")]
    Nix(#[from] nix::Error),
}

/// Trait for types that can execute commands with socket file descriptor handling
pub trait SocketExecutor: Default {
    /// Returns the file descriptor to use for the child process
    fn child_fd(&self) -> i32;

    /// Returns the file descriptor to use for the parent process
    fn parent_fd(&self) -> i32;

    /// Creates a command with the given executable and arguments
    fn command(&self, executable: &str, args: &[&str]) -> Command;
}

/// Executor that uses pkexec for privilege escalation
#[derive(Default)]
pub struct PkexecExecutor;

impl SocketExecutor for PkexecExecutor {
    fn child_fd(&self) -> i32 {
        2
    }

    fn parent_fd(&self) -> i32 {
        4
    }

    fn command(&self, executable: &str, args: &[&str]) -> Command {
        let mut command = Command::new("pkexec");
        command.arg(executable);
        command.args(args);
        command
    }
}

/// Executor that runs commands directly without privilege escalation
#[derive(Default)]
pub struct DirectExecutor;

impl SocketExecutor for DirectExecutor {
    fn child_fd(&self) -> i32 {
        3
    }

    fn parent_fd(&self) -> i32 {
        3
    }

    fn command(&self, executable: &str, args: &[&str]) -> Command {
        let mut command = Command::new(executable);
        command.args(args);
        command
    }
}

/// A unique identifier for a socket address using a UUID
struct AddressIdentifier(uuid::Uuid);

/// A connection to a privileged service, maintaining both the socket and child process
pub struct ServiceConnection {
    /// The Unix domain socket connected to the service
    pub socket: UnixStream,
    pub _child: Pid,
}

impl ServiceConnection {
    /// Creates a new connection to a privileged service using the specified executor
    pub fn new<T: SocketExecutor>(executable: &str, args: &[&str]) -> Result<Self, self::Error> {
        let identity = AddressIdentifier::default();
        let socket_addr = identity.as_unix_address()?;
        let unix_socket = UnixListener::bind_addr(&socket_addr)?;
        unix_socket.set_nonblocking(true)?;

        let exec = T::default();

        let mappings: Vec<FdMapping> = vec![FdMapping {
            parent_fd: unix_socket.into(),
            child_fd: exec.child_fd(),
        }];

        // Emit mappings to log
        for mapping in &mappings {
            tracing::warn!("🔌 mapping: {:?}", mapping);
        }

        match unsafe { nix::unistd::fork() }? {
            nix::unistd::ForkResult::Parent { child } => {
                let socket = UnixStream::connect_addr(&socket_addr)?;
                socket.set_nonblocking(true)?;
                Ok(Self { _child: child, socket })
            }
            nix::unistd::ForkResult::Child => {
                // Ensure we don't leak the listener, so failed pkexec
                // will still result in the listener being closed, and the
                // client connection will fail properly.
                let mut command = exec.command(executable, args);
                command.fd_mappings(mappings)?;
                command.env_remove("PKEXEC_UID");
                let st = command.status()?;
                std::process::exit(st.code().unwrap_or(1));
            }
        }
    }
}

/// An activated service listener that accepts connections from clients
pub struct ServiceListener(pub UnixListener);

impl ServiceListener {
    /// Creates a new service listener using the appropriate executor
    pub fn new() -> io::Result<Self> {
        let server_fd: RawFd = match env::var_os("PKEXEC_UID") {
            Some(_) => PkexecExecutor {}.parent_fd(),
            None => DirectExecutor {}.parent_fd(),
        };
        let listener = unsafe { UnixListener::from(OwnedFd::from_raw_fd(server_fd)) };
        Ok(ServiceListener(listener))
    }
}

impl Deref for ServiceListener {
    type Target = UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for AddressIdentifier {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl AddressIdentifier {
    #[inline]
    fn as_unix_address(&self) -> io::Result<SocketAddr> {
        SocketAddr::from_abstract_name(self.0.as_bytes())
    }
}

/// Initializes a service by handling file descriptor redirection when running under pkexec
pub fn service_init() -> io::Result<()> {
    match env::var_os("PKEXEC_UID") {
        None => Ok(()),
        Some(_) => {
            // Redirect stderr to stdout
            let exec = PkexecExecutor {};
            nix::unistd::dup2(exec.child_fd(), exec.parent_fd())?;
            nix::unistd::close(exec.child_fd())?;
            nix::unistd::dup2(1, exec.child_fd())?;

            Ok(())
        }
    }
}
