//! Jira-Client für den Ticket-Import (ADR 0038): Zugang (Einstellungen + Tresor) und HTTP-Abfragen.

mod client;
mod config;
mod error;

pub use client::JiraClient;
pub use config::{
    JiraCredentials, SECRET_ACCOUNT, SECRET_SERVICE, clear_jira, has_jira_token,
    load_jira_credentials, save_jira,
};
pub use error::JiraError;
