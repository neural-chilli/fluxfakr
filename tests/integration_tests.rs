use assert_cmd::prelude::*;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::thread;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

/// Integration test that launches the built app, lets it run briefly,
/// sends SIGINT, and then verifies the output includes JSON messages and a CSV dump.
#[test]
fn integration_test_market_module_messages_and_app_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn the binary with a slow message rate so that we can capture output easily.
    // Here, we use --module market, --mps 1 (one message per second), and one variant.
    let mut child = Command::cargo_bin("fluxfakr")?
        .args(&["--module", "stock", "--mps", "1", "--variants", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Let the app run for a few seconds so that it produces some output.
    thread::sleep(Duration::from_secs(3));

    // Send SIGINT to the process to trigger graceful shutdown.
    // (This simulates the user pressing Ctrl+C.)
    let pid = child.id() as i32;
    kill(Pid::from_raw(pid), Signal::SIGINT)?;

    // Wait for the process to exit and capture its output.
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // Print combined output for debugging.
    println!("Captured combined output:\n{}", combined_output);

    // Assert that the combined output includes some JSON messages.
    // We expect to see the "instrument" key in the generated JSON messages.
    assert!(
        combined_output.contains("\"instrument\""),
        "Expected output to contain generated JSON messages"
    );
    // Assert that the output includes the CSV dump header.
    assert!(
        combined_output.contains("Generator Internal State Dump"),
        "Expected output to contain the CSV dump (internal state)"
    );

    Ok(())
}
