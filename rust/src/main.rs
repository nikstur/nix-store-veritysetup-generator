use std::env;
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

const SYSTEMD_VERITYSETUP_PATH: &str = std::env!("SYSTEMD_VERITYSETUP_PATH");
const SYSTEMD_ESCAPE_PATH: &str = std::env!("SYSTEMD_ESCAPE_PATH");

/// The name of the service to create
const SERVICE_NAME: &str = "systemd-veritysetup@nix-store.service";

/// The name of the kernel commandline argument
const CMDLINE_ARG_NAME: &str = "storehash";

#[derive(Debug)]
struct Storehash(String);

impl Storehash {
    /// Parse the storehash from a provided kernel commandline
    fn from_cmdline(cmdline: &str) -> Option<Self> {
        let storehash_arg = cmdline
            .split_whitespace()
            .find(|&s| s.contains(&format!("{CMDLINE_ARG_NAME}=")));

        storehash_arg
            .and_then(|s| s.split('=').last())
            .map(|s| Self(String::from(s)))
    }

    fn datadevice(&self) -> Result<String> {
        let data_uuid = convert_to_device_uuid(&self.0[..32])?;
        Ok(format!("/dev/disk/by-partuuid/{data_uuid}"))
    }

    fn hashdevice(&self) -> Result<String> {
        let hash_uuid = convert_to_device_uuid(&self.0[32..])?;
        Ok(format!("/dev/disk/by-partuuid/{hash_uuid}"))
    }
}

impl fmt::Display for Storehash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Convert a UUID from the simple form to the representation udev uses for devices.
///
/// The simple form does not contain hyphens while udev creates devices in `/dev/disk/by-partuuid`
/// with UUIDs that do contains hyphens.
fn convert_to_device_uuid(s: &str) -> Result<String> {
    Ok(Uuid::parse_str(s)
        .with_context(|| format!("Failed to parse {s} as a UUID"))?
        .hyphenated()
        .to_string())
}

/// Escape a string with `systemd-escape`.
fn systemd_escape(s: &str) -> Result<String> {
    // Unwrap this because it's only supposed to fail at build time.
    let mut output = Command::new(SYSTEMD_ESCAPE_PATH)
        .arg(s)
        .output()
        .with_context(|| format!("Failed to run systemd-escape: {SYSTEMD_ESCAPE_PATH}"))?;
    if !output.status.success() {
        return Err(anyhow!("systemd-escape failed"));
    }
    // Remove newline from output
    output.stdout.pop();

    String::from_utf8(output.stdout)
        .context("Failed to convert systemd-escape output to a UTF-8 string")
}

/// Convert a path to a device into a systemd unit name.
///
/// For example: `/dev/vda` -> `dev-vda`
fn convert_to_unit(device_path: &str) -> Result<String> {
    let stripped = device_path
        .strip_prefix('/')
        .with_context(|| format!("Failed to strip '/' from {device_path}"))?;
    Ok(format!(
        "{}.device",
        systemd_escape(stripped).with_context(|| format!("Failed to escape {stripped}"))?
    ))
}

fn create_service_file(storehash: &Storehash) -> Result<String> {
    let datadevice = storehash.datadevice()?;
    let hashdevice = storehash.hashdevice()?;

    let datadevice_unit = convert_to_unit(&datadevice)
        .with_context(|| format!("Failed to convert {datadevice} to systemd unit name."))?;
    let hashdevice_unit = convert_to_unit(&hashdevice)
        .with_context(|| format!("Failed to convert {hashdevice} to systemd unit name."))?;

    let mut buffer = String::new();

    writeln!(
        &mut buffer,
        r#"[Unit]
Description=Integrity Protection Setup for %I
DefaultDependencies=no
IgnoreOnIsolate=true
After=veritysetup-pre.target systemd-udevd-kernel.socket
Before=blockdev@dev-mapper-%i.target
Wants=blockdev@dev-mapper-%i.target
Before=veritysetup.target
BindsTo={datadevice_unit} {hashdevice_unit}
After={datadevice_unit} {hashdevice_unit}"#
    )?;

    writeln!(
        &mut buffer,
        r#"[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart={SYSTEMD_VERITYSETUP_PATH} attach nix-store {datadevice} {hashdevice} {storehash}
ExecStop={SYSTEMD_VERITYSETUP_PATH} detach nix-store"#
    )?;

    Ok(buffer)
}

fn generate() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let destination_dir = &args
        .get(1)
        .context("No command line argument is provided")?;

    let cmdline = fs::read_to_string("/proc/cmdline")?;
    let maybe_storehash = Storehash::from_cmdline(&cmdline);

    let storehash = match maybe_storehash {
        Some(s) => s,
        // If there is no storehash parameter on the cmdline just do nothing.
        None => {
            return Ok(());
        }
    };

    log::info!(
        "Using verity data device {}, hash device {}, and hash {} for nix-store.",
        storehash.datadevice()?,
        storehash.hashdevice()?,
        storehash,
    );

    let service_file = create_service_file(&storehash)?;

    // Write service to destination directory
    let service_file_path = format!("{destination_dir}/{SERVICE_NAME}");
    let mut file = File::create(&service_file_path).context("Failed to create service file")?;
    file.write_all(service_file.as_bytes())?;

    // Add a symlink to destination directory so that the created unit is pulled into the
    // transaction
    fs::create_dir(format!("{destination_dir}/veritysetup.target.requires"))?;
    std::os::unix::fs::symlink(
        service_file_path,
        format!("{destination_dir}/veritysetup.target.requires/{SERVICE_NAME}"),
    )?;

    Ok(())
}

fn main() {
    kernlog::init().expect("Failed to initialize kernel logger");

    if let Err(e) = generate() {
        log::error!("{e:#}");
        std::process::exit(1);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use expect_test::expect;

    #[test]
    fn parse_storehash_from_cmdline() {
        let expected_storehash = "94821122dbec8355df07f3670177b0cb147683a355c07da6a2fb85313cc02254";
        let cmdline = format!("{CMDLINE_ARG_NAME}={expected_storehash}");
        let storehash = Storehash::from_cmdline(&cmdline).unwrap();
        assert_eq!(storehash.0, expected_storehash);
    }

    #[test]
    fn write_service_unit() {
        let storehash = Storehash::from_cmdline(&format!(
            "{CMDLINE_ARG_NAME}=94821122dbec8355df07f3670177b0cb147683a355c07da6a2fb85313cc02254"
        ))
        .unwrap();
        let actual_service_file = create_service_file(&storehash).unwrap();

        let expected_service_file = expect![[r#"
            [Unit]
            Description=Integrity Protection Setup for %I
            DefaultDependencies=no
            IgnoreOnIsolate=true
            After=veritysetup-pre.target systemd-udevd-kernel.socket
            Before=blockdev@dev-mapper-%i.target
            Wants=blockdev@dev-mapper-%i.target
            Before=veritysetup.target
            BindsTo=dev-disk-by\x2dpartuuid-94821122\x2ddbec\x2d8355\x2ddf07\x2df3670177b0cb.device dev-disk-by\x2dpartuuid-147683a3\x2d55c0\x2d7da6\x2da2fb\x2d85313cc02254.device
            After=dev-disk-by\x2dpartuuid-94821122\x2ddbec\x2d8355\x2ddf07\x2df3670177b0cb.device dev-disk-by\x2dpartuuid-147683a3\x2d55c0\x2d7da6\x2da2fb\x2d85313cc02254.device
            [Service]
            Type=oneshot
            RemainAfterExit=yes
            ExecStart=systemd-veritysetup attach nix-store /dev/disk/by-partuuid/94821122-dbec-8355-df07-f3670177b0cb /dev/disk/by-partuuid/147683a3-55c0-7da6-a2fb-85313cc02254 94821122dbec8355df07f3670177b0cb147683a355c07da6a2fb85313cc02254
            ExecStop=systemd-veritysetup detach nix-store
        "#]];

        expected_service_file.assert_eq(&actual_service_file);
    }
}
