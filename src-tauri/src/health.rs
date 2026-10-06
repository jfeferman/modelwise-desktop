//! What a connection's state means, in words a person can act on. Decided
//! here, once, because the tray icon needs it with no panel open.

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Health {
    Working,
    Attention,
    Broken,
}

impl Health {
    pub fn name(self) -> &'static str {
        match self {
            Health::Working => "working",
            Health::Attention => "attention",
            Health::Broken => "broken",
        }
    }
}

/// A connection's health, and what is wrong, most serious first.
pub fn verdict(connection: &Value) -> (Health, Vec<String>) {
    let check = |key: &str| connection.pointer(&format!("/check/{key}")).and_then(Value::as_str);
    let mut broken = Vec::new();
    let mut attention = Vec::new();

    if check("token") == Some("revoked") {
        broken.push("Its token was revoked. Connect again.".to_string());
    }

    match check("settings") {
        Some("missing") => broken.push("Claude Code is not set up to send to Modelwise.".to_string()),
        Some("drifted") => broken.push("Claude Code's telemetry settings were changed.".to_string()),
        Some("unreadable") => broken.push("Claude Code's settings.json cannot be read.".to_string()),
        _ => {}
    }

    if check("server") == Some("unreachable") {
        attention.push("Modelwise cannot be reached right now.".to_string());
    }

    if let Some(error) = connection.pointer("/lastSync/error").and_then(Value::as_str) {
        attention.push(format!("The last sync failed: {error}"));
    }

    let health = if !broken.is_empty() {
        Health::Broken
    } else if !attention.is_empty() {
        Health::Attention
    } else {
        Health::Working
    };

    broken.extend(attention);
    (health, broken)
}

/// Adds `health` and `problems` to each connection of a status document, and
/// returns the worst health among them. No connection at all is `Working`:
/// nothing is wrong, there is just nothing yet.
pub fn annotate(status: &mut Value) -> Health {
    let mut worst = Health::Working;

    if let Some(connections) = status.get_mut("connections").and_then(Value::as_array_mut) {
        for connection in connections {
            let (health, problems) = verdict(connection);
            worst = worst.max(health);
            connection["health"] = json!(health.name());
            connection["problems"] = json!(problems);
        }
    }

    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection(check: Value, last_sync: Value) -> Value {
        json!({ "configDir": "/x", "check": check, "lastSync": last_sync })
    }

    #[test]
    fn a_working_connection_has_nothing_to_say() {
        let (health, problems) = verdict(&connection(
            json!({ "settings": "ok", "server": "reachable", "token": "live" }),
            json!({ "at": "2026-10-06T00:00:00Z", "sent": 3, "error": null }),
        ));
        assert_eq!(health, Health::Working);
        assert!(problems.is_empty());
    }

    #[test]
    fn broken_outranks_attention_and_comes_first() {
        let (health, problems) = verdict(&connection(
            json!({ "settings": "missing", "server": "unreachable", "token": "unknown" }),
            json!({ "at": "2026-10-06T00:00:00Z", "sent": 0, "error": "refused" }),
        ));
        assert_eq!(health, Health::Broken);
        assert_eq!(problems.len(), 3);
        assert!(problems[0].contains("not set up"));
        assert!(problems[2].contains("refused"));
    }

    #[test]
    fn a_status_without_check_is_judged_on_its_sync_alone() {
        let (health, _) = verdict(&json!({ "configDir": "/x", "lastSync": { "error": "x" } }));
        assert_eq!(health, Health::Attention);
    }

    #[test]
    fn annotating_reports_the_worst_connection() {
        let mut status = json!({ "schema": 1, "connections": [
            connection(json!({ "settings": "ok", "server": "reachable", "token": "live" }), Value::Null),
            connection(json!({ "settings": "ok", "server": "reachable", "token": "revoked" }), Value::Null)
        ]});
        assert_eq!(annotate(&mut status), Health::Broken);
        assert_eq!(status["connections"][0]["health"], "working");
        assert_eq!(status["connections"][1]["problems"][0], "Its token was revoked. Connect again.");

        assert_eq!(annotate(&mut json!({ "schema": 1, "connections": [] })), Health::Working);
    }
}
