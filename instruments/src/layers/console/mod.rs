use colored::{ColoredString, Colorize};
use opentelemetry::logs::Severity;

pub(crate) mod logs;

pub(crate) fn severity_to_str(severity: Option<Severity>) -> ColoredString {
    match severity {
        Some(Severity::Trace) | Some(Severity::Trace2) | Some(Severity::Trace3) | Some(Severity::Trace4) => {
            "TRACE".magenta()
        }

        Some(Severity::Debug) | Some(Severity::Debug2) | Some(Severity::Debug3) | Some(Severity::Debug4) => {
            "DEBUG".cyan()
        }

        Some(Severity::Info) | Some(Severity::Info2) | Some(Severity::Info3) | Some(Severity::Info4) => {
            "INFO".blue()
        }

        Some(Severity::Warn) | Some(Severity::Warn2) | Some(Severity::Warn3) | Some(Severity::Warn4) => {
            "WARN".yellow()
        }

        Some(Severity::Error) | Some(Severity::Error2) | Some(Severity::Error3) | Some(Severity::Error4) => {
            "ERROR".red()
        }

        Some(Severity::Fatal) | Some(Severity::Fatal2) | Some(Severity::Fatal3) | Some(Severity::Fatal4) => {
            "FATAL".red()
        }

        None => "UNKNOWN".dimmed(),
    }
}
