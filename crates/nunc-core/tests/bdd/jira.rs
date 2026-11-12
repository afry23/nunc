//! Steps für `features/jira/jira_core.feature` (Jira-Import, ADR 0038).

use cucumber::{given, then, when};
use nunc_core::jira::{
    JiraConfig, JiraEndpoint, JiraIssue, JiraKind, api_path, escape_jql, is_imported, issue_url,
    parse_search, search_jql, todo_line,
};
use nunc_core::model::Todo;

use crate::DomainWorld;

#[derive(Debug, Default)]
pub struct State {
    text: String,
    todos: Vec<Todo>,
    hits: Option<Result<Vec<JiraIssue>, String>>,
}

/// Steuerzeichen und Anführungszeichen, die in Feature-Dateien nicht direkt stehen können.
fn decode(s: &str) -> String {
    s.replace("{Q}", "\"")
        .replace("{B}", "\\")
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
}

fn issue(key: &str, summary: &str, base: &str) -> JiraIssue {
    JiraIssue {
        key: key.into(),
        summary: summary.into(),
        url: issue_url(base, key),
    }
}

fn kind(s: &str) -> JiraKind {
    s.parse().expect("Jira-Art")
}

#[when(expr = "I build the Jira query for {string}")]
fn build_query(world: &mut DomainWorld, input: String) {
    world.jira.text = search_jql(&decode(&input));
}

#[then(expr = "the Jira query is {string}")]
fn query_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.jira.text, expected);
}

#[when(expr = "I escape {string} for JQL")]
fn escape(world: &mut DomainWorld, input: String) {
    world.jira.text = escape_jql(&decode(&input));
}

#[then(expr = "the escaped text is {string}")]
fn escaped_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.jira.text, decode(&expected));
}

#[when(expr = "I build the URL of {string} on {string}")]
fn build_url(world: &mut DomainWorld, key: String, base: String) {
    world.jira.text = issue_url(&base, &key);
}

#[then(expr = "the Jira URL is {string}")]
fn url_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.jira.text, expected);
}

#[when(expr = "I build the todo line for the ticket {string} {string} on {string}")]
fn build_line(world: &mut DomainWorld, key: String, summary: String, base: String) {
    world.jira.text = todo_line(&issue(&key, &summary, &base));
}

#[when(expr = "I build the todo line for a ticket with the summary {string}")]
fn build_line_for_summary(world: &mut DomainWorld, summary: String) {
    world.jira.text = todo_line(&issue("PROJ-1", &decode(&summary), "https://x.net"));
}

#[then(expr = "the Jira todo line is {string}")]
fn line_is(world: &mut DomainWorld, expected: String) {
    assert_eq!(world.jira.text, expected);
}

#[when(expr = "I parse the Jira response for {string}:")]
fn parse(world: &mut DomainWorld, base: String, step: &cucumber::gherkin::Step) {
    let body = step.docstring.as_ref().expect("Docstring fehlt");
    world.jira.hits = Some(parse_search(body, &base).map_err(|e| e.to_string()));
}

#[when(expr = "I parse a Jira response with {int} tickets")]
fn parse_many(world: &mut DomainWorld, count: usize) {
    let issues: Vec<String> = (1..=count)
        .map(|n| format!(r#"{{"key":"PROJ-{n}","fields":{{"summary":"Ticket {n}"}}}}"#))
        .collect();
    let body = format!(r#"{{"issues":[{}]}}"#, issues.join(","));
    world.jira.hits = Some(parse_search(&body, "https://x.net").map_err(|e| e.to_string()));
}

fn hits(world: &DomainWorld) -> &Vec<JiraIssue> {
    world
        .jira
        .hits
        .as_ref()
        .expect("nicht geparst")
        .as_ref()
        .expect("Antwort abgelehnt")
}

#[then(expr = "I get {int} Jira tickets")]
fn hit_count(world: &mut DomainWorld, count: usize) {
    assert_eq!(hits(world).len(), count);
}

#[then(expr = "ticket {int} is {string} {string} at {string}")]
fn hit_is(world: &mut DomainWorld, n: usize, key: String, summary: String, url: String) {
    assert_eq!(hits(world)[n - 1], JiraIssue { key, summary, url });
}

#[then("the Jira response is rejected")]
fn rejected(world: &mut DomainWorld) {
    let result = world.jira.hits.as_ref().expect("nicht geparst");
    assert!(result.is_err(), "unerwartet akzeptiert: {result:?}");
}

#[given("the Jira todos:")]
fn given_todos(world: &mut DomainWorld, step: &cucumber::gherkin::Step) {
    let table = step.table.as_ref().expect("Tabelle fehlt");
    world.jira.todos = table
        .rows
        .iter()
        .skip(1)
        .enumerate()
        .map(|(i, row)| Todo {
            id: i as i32 + 1,
            raw_text: row[0].clone(),
            priority: None,
            completed: row[1] == "true",
            completed_at: None,
            created_at: None,
            description: row[0].clone(),
            project_tags: vec![],
            context_tags: vec![],
            pomodoro_count: 0,
            position: i as u32,
            parent_id: None,
        })
        .collect();
}

fn imported(world: &DomainWorld, key: &str) -> bool {
    is_imported(&world.jira.todos, &issue(key, "", "https://x.net"))
}

#[then(expr = "the ticket {string} is imported")]
fn is_imported_step(world: &mut DomainWorld, key: String) {
    assert!(imported(world, &key));
}

#[then(expr = "the ticket {string} is not imported")]
fn is_not_imported(world: &mut DomainWorld, key: String) {
    assert!(!imported(world, &key));
}

#[then(expr = "the {string} path for {string} is {string}")]
fn path_is(_: &mut DomainWorld, endpoint: String, kind_name: String, expected: String) {
    let endpoint = match endpoint.as_str() {
        "search" => JiraEndpoint::Search,
        "myself" => JiraEndpoint::Myself,
        other => panic!("unbekannter Endpunkt {other}"),
    };
    assert_eq!(api_path(kind(&kind_name), endpoint), expected);
}

#[then(
    expr = "the Jira configuration of kind {string} with URL {string} and e-mail {string} is {word}"
)]
fn config_state(_: &mut DomainWorld, kind_name: String, url: String, email: String, state: String) {
    let cfg = JiraConfig {
        base_url: url,
        kind: kind(&kind_name),
        email,
    };
    assert_eq!(cfg.is_complete(), state == "complete");
}
