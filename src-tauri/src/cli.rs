//! Runs the embedded `modelwise` command and reads what `--json` prints.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

/// The version of the command's JSON this app was written against. The command
/// raises its number whenever a shape changes, and the app then refuses to
/// guess at what it is reading.
pub const SCHEMA: u64 = 1;

/// The command sits beside the app's own executable: in the bundle
/// (`Contents/MacOS`), and in `target/` when developing.
fn executable() -> Result<PathBuf, String> {
    let app = std::env::current_exe().map_err(|error| format!("Could not find the app's own path: {error}"))?;
    let name = if cfg!(windows) { "modelwise.exe" } else { "modelwise" };
    let path = app.with_file_name(name);

    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("The modelwise command is missing from the app ({})", path.display()))
    }
}

/// Runs `modelwise <args> --json` and returns the one document it prints. A
/// failure the command reports becomes its message.
pub fn run(args: &[&str]) -> Result<Value, String> {
    let output = Command::new(executable()?)
        .args(args)
        .arg("--json")
        // Bun colours errors when this is set, whoever is reading.
        .env_remove("FORCE_COLOR")
        .output()
        .map_err(|error| format!("Could not run the modelwise command: {error}"))?;

    document(&String::from_utf8_lossy(&output.stdout))
}

fn document(printed: &str) -> Result<Value, String> {
    let value: Value = serde_json::from_str(printed.trim())
        .map_err(|_| "The modelwise command printed something this app cannot read.".to_string())?;

    match value.get("schema").and_then(Value::as_u64) {
        Some(SCHEMA) => {}
        Some(other) => {
            return Err(format!(
                "The modelwise command speaks version {other} of its output and this app reads version {SCHEMA}. Update the app."
            ))
        }
        None => return Err("The modelwise command printed something this app cannot read.".to_string()),
    }

    if let Some(message) = value.pointer("/error/message").and_then(Value::as_str) {
        return Err(message.to_string());
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_at_this_schema_is_passed_on() {
        let value = document(r#"{"schema":1,"connections":[]}"#).unwrap();
        assert_eq!(value["connections"], serde_json::json!([]));
    }

    #[test]
    fn a_failure_becomes_its_message() {
        let error = document(r#"{"schema":1,"error":{"code":"not-connected","message":"Not connected."}}"#).unwrap_err();
        assert_eq!(error, "Not connected.");
    }

    #[test]
    fn another_schema_is_refused_rather_than_guessed_at() {
        assert!(document(r#"{"schema":2,"connections":[]}"#).unwrap_err().contains("version 2"));
        assert!(document(r#"{"connections":[]}"#).is_err());
        assert!(document("Usage: modelwise <command>").is_err());
    }
}
