//! Regression checks for SafeRustXNG's public names and renamed Cargo binaries.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use saferustxng::settings::Settings;
use saferustxng::version::{VERSION, version_string};

#[test]
fn default_settings_use_the_fork_name() {
    assert_eq!(Settings::default().general.instance_name, "SafeRustXNG");
    assert_eq!(Settings::defaults().general.instance_name, "SafeRustXNG");
}

#[test]
fn server_binary_reports_the_fork_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_saferustxng"))
        .arg("--version")
        .output()
        .expect("run the renamed server binary");
    assert!(output.status.success());
    assert_eq!(version_string(), format!("SafeRustXNG {VERSION}"));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        version_string()
    );
}

#[test]
fn server_help_uses_the_renamed_command_and_idle_timeout_variable() {
    let output = Command::new(env!("CARGO_BIN_EXE_saferustxng"))
        .arg("--help")
        .output()
        .expect("run the renamed server binary");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.starts_with("SafeRustXNG -"));
    assert!(help.contains("saferustxng [OPTIONS]"));
    assert!(help.contains("SAFERUSTXNG_IDLE_TIMEOUT"));
}

#[test]
fn renamed_idle_timeout_variable_stops_an_unattended_server() {
    let directory = tempfile::tempdir().expect("create isolated server settings");
    let settings_path = directory.path().join("settings.toml");
    let mut settings = Settings::defaults();
    settings.use_default_settings = Some(false);
    settings.engines.clear();
    settings.server.idle_timeout = 0;
    std::fs::write(&settings_path, toml::to_string(&settings).unwrap()).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_saferustxng"));
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy().to_ascii_uppercase();
        if name.starts_with("SEARXNG_") || name.starts_with("SAFERUSTXNG_") {
            command.env_remove(key);
        }
    }
    let mut child = command
        .arg("--config")
        .arg(&settings_path)
        .args(["--bind", "127.0.0.1", "--port", "0"])
        .env("SAFERUSTXNG_IDLE_TIMEOUT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start the renamed server with an idle timeout");
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("check the test server") {
            assert!(status.success());
            assert!(started.elapsed() >= Duration::from_secs(1));
            break;
        }
        if started.elapsed() >= Duration::from_secs(15) {
            child.kill().expect("stop the test server after timeout");
            child.wait().expect("reap the test server");
            panic!("SAFERUSTXNG_IDLE_TIMEOUT did not stop the test server");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn index_binary_reports_the_renamed_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_saferustxng-index"))
        .output()
        .expect("run the renamed read-only index binary");
    assert_eq!(output.status.code(), Some(2));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("usage: saferustxng-index <search|stats>"));
}
