pub mod logs;

use colored::{ColoredString, Colorize};
use opentelemetry::logs::Severity;

pub fn severity_to_str(severity: Option<Severity>) -> ColoredString {
    match severity {
        Some(Severity::Trace | Severity::Trace2 | Severity::Trace3 | Severity::Trace4) => "TRACE".magenta(),

        Some(Severity::Debug | Severity::Debug2 | Severity::Debug3 | Severity::Debug4) => "DEBUG".cyan(),

        Some(Severity::Info | Severity::Info2 | Severity::Info3 | Severity::Info4) => "INFO".blue(),

        Some(Severity::Warn | Severity::Warn2 | Severity::Warn3 | Severity::Warn4) => "WARN".yellow(),

        Some(Severity::Error | Severity::Error2 | Severity::Error3 | Severity::Error4) => "ERROR".red(),

        Some(Severity::Fatal | Severity::Fatal2 | Severity::Fatal3 | Severity::Fatal4) => "FATAL".red(),

        None => "UNKNOWN".dimmed(),
    }
}
