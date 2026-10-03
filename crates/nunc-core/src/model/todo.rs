use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Ein todo.txt-Eintrag.
///
/// `description` enthält wie in der Go-App die Tags (`Bericht +work @desk`); `project_tags` und
/// `context_tags` behalten ihr Präfix (`+work`, `@desk`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: i32,
    pub raw_text: String,
    /// `A`–`Z`, `null` wenn keine Priorität gesetzt ist.
    pub priority: Option<char>,
    pub completed: bool,
    pub completed_at: Option<NaiveDate>,
    pub created_at: Option<NaiveDate>,
    pub description: String,
    pub project_tags: Vec<String>,
    pub context_tags: Vec<String>,
    pub pomodoro_count: u32,
    pub position: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_camel_case_json() {
        let json = r#"{
            "id": 4, "rawText": "x 2026-09-25 2026-09-18 Plan sprint +work", "priority": null,
            "completed": true, "completedAt": "2026-09-25", "createdAt": "2026-09-18",
            "description": "Plan sprint +work", "projectTags": ["+work"], "contextTags": [],
            "pomodoroCount": 3, "position": 4
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert!(todo.completed && todo.priority.is_none());
        assert_eq!(todo.completed_at, NaiveDate::from_ymd_opt(2026, 9, 25));
        assert_eq!(todo.created_at, NaiveDate::from_ymd_opt(2026, 9, 18));
        assert_eq!(todo.project_tags, ["+work"]);

        let value = serde_json::to_value(&todo).unwrap();
        assert_eq!(value["completedAt"], "2026-09-25");
        assert_eq!(value["pomodoroCount"], 3);
    }

    #[test]
    fn priority_is_a_single_letter() {
        let json = r#"{
            "id": 1, "rawText": "(A) x", "priority": "A", "completed": false,
            "completedAt": null, "createdAt": null, "description": "x",
            "projectTags": [], "contextTags": [], "pomodoroCount": 0, "position": 1
        }"#;
        let todo: Todo = serde_json::from_str(json).unwrap();
        assert_eq!(todo.priority, Some('A'));
    }
}
