use crate::proto::SystemdAction;

use failure::{format_err, Error};
use log::{info, warn};

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const SYSTEMD_DIR: &str = "/etc/systemd/system/";

pub fn service_name(port: i64) -> String {
    format!("cache-{}.service", port)
}

pub fn do_action(action: SystemdAction, port: i64) -> Result<(), Error> {
    match action {
        SystemdAction::Setup => reload(),
        SystemdAction::Remove => {
            let sname = service_name(port);
            if let Err(err) = do_action(SystemdAction::Stop, port) {
                warn!("fail to stop cache-service {} due error {:?}", sname, err);
            }

            let mut pb = PathBuf::from(SYSTEMD_DIR);
            pb.push(&sname);

            if let Err(err) = fs::remove_file(pb.as_path()) {
                warn!("fail to delete cache service file {} due {}", sname, err);
            }
            do_action(SystemdAction::Setup, port)
        }
        _ => {
            if port < 0 {
                return Err(format_err!(
                    "{:?} must run with an avaliable port but given {}",
                    action,
                    port
                ));
            }
            do_systemd(action_to_str(action), port)
        }
    }
}

fn action_to_str(action: SystemdAction) -> &'static str {
    match action {
        SystemdAction::Restart => "restart",
        SystemdAction::Start => "start",
        SystemdAction::Stop => "stop",
        _ => unreachable!(),
    }
}

fn do_systemd(cmd: &str, port: i64) -> Result<(), Error> {
    let args = [cmd, &service_name(port)];
    call_systemd(&args)
}

fn reload() -> Result<(), Error> {
    call_systemd(&["daemon-reload"; 1])
}

fn call_systemd(args: &[&str]) -> Result<(), Error> {
    let child = Command::new("systemctl")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    info!(
        "execute systemctl {} exit with {} and stdout: {} stderr: {}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
