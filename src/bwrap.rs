use libmount::Overlay;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempdir::TempDir;

use crate::oci::pull_image;

struct OverlayMount {
    work_dir: TempDir,
    upper_dir: TempDir,
    mount_dir: PathBuf,
}

impl OverlayMount {
    fn new(root: PathBuf) -> Self {
        let work_dir = TempDir::new_in("tmpdir", "bwrap-work").unwrap();
        let upper_dir = TempDir::new_in("tmpdir", "bwrap-upper").unwrap();
        let mount_dir = PathBuf::from("tmpdir/bwrap-mount");

        let cmd = Command::new("fuse-overlayfs")
            .arg("-o")
            .arg("allow_root")
            .arg("-o")
            .arg(format!("lowerdir={}", root.to_str().unwrap()))
            .arg("-o")
            .arg(format!("upperdir={}", upper_dir.path().to_str().unwrap()))
            .arg("-o")
            .arg(format!("workdir={}", work_dir.path().to_str().unwrap()))
            .arg(mount_dir.to_str().unwrap())
            .spawn()
            .expect("failed to execute process");

        cmd.wait_with_output().expect("failed to wait on child");

        Self {
            work_dir,
            upper_dir,
            mount_dir,
        }
    }
}

impl Drop for OverlayMount {
    fn drop(&mut self) {
        Command::new("umount")
            .arg(self.mount_dir.to_str().unwrap())
            .output()
            .expect("failed to execute process");
    }
}

#[test]
fn test_overlay_mount() {
    let name = "registry.hub.docker.com/library/alpine:latest";
    let image_dir = pull_image(name, false).unwrap();
    let mount = OverlayMount::new(image_dir);

    // println!("{:?}", mount.mount_point);
}

struct SandboxOptions {
    root: OverlayMount,
    network: bool,
    env: BTreeMap<String, String>,
    unset_env: Vec<String>,
    chdir: Option<PathBuf>,

    devices: Vec<String>,

    bind_dev: BTreeMap<String, String>,
    bind: BTreeMap<String, String>,
    bind_ro: BTreeMap<String, String>,
}

fn get_bwrap_args(sandbox: &SandboxOptions) -> Vec<&str> {
    let mut args = vec![
        "--symlink",
        "usr/bin",
        "/bin",
        "--symlink",
        "usr/sbin",
        "/sbin",
        "--symlink",
        "usr/lib",
        "/lib",
        "--symlink",
        "usr/lib64",
        "/lib64",
        "--bind",
        sandbox.root.mount_dir.to_str().unwrap(),
        "/",
        "--dir",
        "/var",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
        "--unshare-all",
        "--die-with-parent",
        // "--uid",
        // "0",
        // "--gid",
        // "0",
        "--exec-label",
        "system_u:system_r:container_t:s0:c1,c2",
        "--cap-add",
        "CAP_SYS_ADMIN",
    ];

    if sandbox.network {
        args.extend_from_slice(&[
            "--share-net",
            "--ro-bind",
            "/etc/resolv.conf",
            "/etc/resolv.conf",
        ]);
    }

    for (key, value) in &sandbox.env {
        args.extend_from_slice(&["--setenv", key, value]);
    }

    args
    // let mut command = Command::new("bwrap");
}

fn run_sandbox(sandbox: &SandboxOptions, sandbox_command: Vec<&str>) {
    let args = get_bwrap_args(sandbox);
    let mut command = Command::new("bwrap");
    command.args(args);
    command.args(sandbox_command);
    let child = command.spawn().unwrap();

    child.wait_with_output().unwrap();
}

pub fn test_bwrap() {
    let name = "registry.hub.docker.com/library/fedora:latest";
    let image_dir = pull_image(name, false).unwrap();
    let mount = OverlayMount::new(PathBuf::from("tmpdir/chroot"));

    let sandbox = SandboxOptions {
        root: mount,
        network: false,
        env: BTreeMap::from([(
            "PATH".to_string(),
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
        )]),
        unset_env: Vec::new(),
        chdir: None,
        devices: Vec::new(),
        bind_dev: BTreeMap::new(),
        bind: BTreeMap::new(),
        bind_ro: BTreeMap::new(),
    };

    run_sandbox(&sandbox, vec!["/bin/bash"]);
}
