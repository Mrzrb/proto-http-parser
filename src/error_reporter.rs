//! Error reporting and diagnostic system
//!
//! This module provides comprehensive error reporting with detailed context,
//! suggestions, and recovery mechanisms.

use crate::core::{
    errors::*,
    SourceLocation, ValidationSuggestion,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Comprehensive error reporter with context and suggestions
pub struct ErrorReporter {
    /// Configuration for error reporting
    config: ErrorReporterConfig,
    /// Source file content cache for context generation
    source_cache: HashMap<PathBuf, String>,
    /// Error statistics
    stats: ErrorStats,
}

/// Configuration for error reporting behavior
#[derive(Debug, Clone)]
pub struct ErrorReporterConfig {
    /// Number of context lines to show around errors
    pub context_lines: usize,
    /// Whether to show suggestions
    pub show_suggestions: bool,
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Maximum width for error messages
    pub max_width: usize,
    /// Whether to show error codes
    pub show_error_codes: bool,
}

impl Default for ErrorReporterConfig {
    fn default() -> Self {
        Self {
            context_lines: 2,
            show_suggestions: true,
            use_colors: true,
            max_width: 120,
            show_error_codes: true,
        }
    }
}

/// Error statistics for reporting
#[derive(Debug, Default)]
pub struct ErrorStats {
    /// Total number of errors
    pub total_errors: usize,
    /// Errors by category
    pub errors_by_type: HashMap<String, usize>,
    /// Files with errors
    pub files_with_errors: HashMap<PathBuf, usize>,
}

/// Detailed error report with context and suggestions
#[derive(Debug)]
pub struct DetailedErrorReport {
    /// The original error
    pub error: ProtoHttpParserError,
    /// Error location in source
    pub location: Option<SourceLocation>,
    /// Source context around the error
    pub context: Option<SourceContext>,
    /// Suggestions for fixing the error
    pub suggestions: Vec<ValidationSuggestion>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error code for documentation lookup
    pub error_code: Option<String>,
}

/// Source context around an error
#[derive(Debug)]
pub struct SourceContext {
    /// Lines before the error
    pub before_lines: Vec<ContextLine>,
    /// The line containing the error
    pub error_line: ContextLine,
    /// Lines after the error
    pub after_lines: Vec<ContextLine>,
    /// Column range highlighting the error
    pub error_range: Option<(usize, usize)>,
}

/// A line of source code with context
#[derive(Debug)]
pub struct ContextLine {
    /// Line number (1-based)
    pub line_number: usize,
    /// Line content
    pub content: String,
    /// Whether this line contains the error
    pub is_error_line: bool,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Fatal error that prevents compilation
    Error,
    /// Warning that should be addressed
    Warning,
    /// Informational message
    Info,
    /// Hint for improvement
    Hint,
}

impl ErrorReporter {
    /// Create a new error reporter with default configuration
    pub fn new() -> Self {
        Self::with_config(ErrorReporterConfig::default())
    }

    /// Create a new error reporter with custom configuration
    pub fn with_config(config: ErrorReporterConfig) -> Self {
        Self {
            config,
            source_cache: HashMap::new(),
            stats: ErrorStats::default(),
        }
    }

    /// Add source file content to the cache for context generation
    pub fn add_source_file(&mut self, path: PathBuf, content: String) {
        self.source_cache.insert(path, content);
    }

    /// Generate a detailed error report
    pub fn generate_report(&mut self, error: ProtoHttpParserError) -> DetailedErrorReport {
        self.stats.total_errors += 1;
        
        let error_type = self.get_error_type(&error);
        *self.stats.errors_by_type.entry(error_type.clone()).or_insert(0) += 1;

        let location = self.extract_location(&error);
        let context = self.generate_context(&location);
        let suggestions = self.generate_suggestions(&error);
        let severity = self.determine_severity(&error);
        let error_code = self.get_error_code(&error);

        if let Some(ref loc) = location {
            if let Some(ref file) = loc.file {
                *self.stats.files_with_errors.entry(file.clone()).or_insert(0) += 1;
            }
        }

        DetailedErrorReport {
            error,
            location,
            context,
            suggestions,
            severity,
            error_code,
        }
    }

    /// Format an error report as a human-readable string
    pub fn format_report(&self, report: &DetailedErrorReport) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&self.format_error_header(report));
        output.push('\n');

        // Source context
        if let Some(ref context) = report.context {
            output.push_str(&self.format_source_context(context));
            output.push('\n');
        }

        // Error details
        output.push_str(&self.format_error_details(report));
        output.push('\n');

        // Suggestions
        if self.config.show_suggestions && !report.suggestions.is_empty() {
            output.push_str(&self.format_suggestions(&report.suggestions));
            output.push('\n');
        }

        output
    }

    /// Format multiple error reports
    pub fn format_multiple_reports(&self, reports: &[DetailedErrorReport]) -> String {
        let mut output = String::new();

        // Summary header
        output.push_str(&format!("Found {} error(s):\n\n", reports.len()));

        // Individual reports
        for (i, report) in reports.iter().enumerate() {
            output.push_str(&format!("Error {}: ", i + 1));
            output.push_str(&self.format_report(report));
            if i < reports.len() - 1 {
                output.push('\n');
            }
        }

        // Summary footer
        output.push_str(&self.format_summary(&self.stats));

        output
    }

    /// Extract location information from an error
    fn extract_location(&self, error: &ProtoHttpParserError) -> Option<SourceLocation> {
        match error {
            ProtoHttpParserError::Parse(ParseError::Syntax { line, column, .. }) => {
                Some(SourceLocation {
                    file: None,
                    line: *line,
                    column: *column,
                    length: None,
                })
            }
            ProtoHttpParserError::Parse(ParseError::UnexpectedToken { line, .. }) => {
                Some(SourceLocation {
                    file: None,
                    line: *line,
                    column: 1,
                    length: None,
                })
            }
            ProtoHttpParserError::Validation(ValidationError::UndefinedType { line, .. }) => {
                Some(SourceLocation {
                    file: None,
                    line: *line,
                    column: 1,
                    length: None,
                })
            }
            ProtoHttpParserError::Validation(ValidationError::DuplicateDefinition { line, .. }) => {
                Some(SourceLocation {
                    file: None,
                    line: *line,
                    column: 1,
                    length: None,
                })
            }
            ProtoHttpParserError::Validation(ValidationError::InvalidHttpAnnotation { line, .. }) => {
                Some(SourceLocation {
                    file: None,
                    line: *line,
                    column: 1,
                    length: None,
                })
            }
            _ => None,
        }
    }

    /// Generate source context around an error location
    fn generate_context(&self, location: &Option<SourceLocation>) -> Option<SourceContext> {
        let location = location.as_ref()?;
        let file = location.file.as_ref()?;
        let content = self.source_cache.get(file)?;

        let lines: Vec<&str> = content.lines().collect();
        if location.line == 0 || location.line > lines.len() {
            return None;
        }

        let error_line_idx = location.line - 1; // Convert to 0-based
        let start_idx = error_line_idx.saturating_sub(self.config.context_lines);
        let end_idx = std::cmp::min(error_line_idx + self.config.context_lines + 1, lines.len());

        let mut before_lines = Vec::new();
        let mut after_lines = Vec::new();

        for i in start_idx..error_line_idx {
            before_lines.push(ContextLine {
                line_number: i + 1,
                content: lines[i].to_string(),
                is_error_line: false,
            });
        }

        let error_line = ContextLine {
            line_number: location.line,
            content: lines[error_line_idx].to_string(),
            is_error_line: true,
        };

        for i in (error_line_idx + 1)..end_idx {
            after_lines.push(ContextLine {
                line_number: i + 1,
                content: lines[i].to_string(),
                is_error_line: false,
            });
        }

        let error_range = if location.column > 0 {
            let start_col = location.column.saturating_sub(1);
            let end_col = start_col + location.length.unwrap_or(1);
            Some((start_col, end_col))
        } else {
            None
        };

        Some(SourceContext {
            before_lines,
            error_line,
            after_lines,
            error_range,
        })
    }

    /// Generate suggestions for fixing an error
    fn generate_suggestions(&self, error: &ProtoHttpParserError) -> Vec<ValidationSuggestion> {
        let mut suggestions = Vec::new();

        match error {
            ProtoHttpParserError::Parse(ParseError::UnexpectedToken { token, expected, .. }) => {
                suggestions.push(ValidationSuggestion {
                    issue_type: "UnexpectedToken".to_string(),
                    message: format!("Replace '{}' with '{}'", token, expected),
                    suggested_fix: Some(expected.clone()),
                    confidence: 0.8,
                    location: None,
                });
            }
            ProtoHttpParserError::Validation(ValidationError::UndefinedType { type_name, .. }) => {
                suggestions.push(ValidationSuggestion {
                    issue_type: "UndefinedType".to_string(),
                    message: format!("Define the type '{}' or check import statements", type_name),
                    suggested_fix: None,
                    confidence: 0.7,
                    location: None,
                });
            }
            ProtoHttpParserError::Validation(ValidationError::InvalidHttpAnnotation { message, .. }) => {
                if message.contains("must start with '/'") {
                    suggestions.push(ValidationSuggestion {
                        issue_type: "InvalidHttpAnnotation".to_string(),
                        message: "Add a leading '/' to the path template".to_string(),
                        suggested_fix: None,
                        confidence: 0.9,
                        location: None,
                    });
                }
            }
            _ => {}
        }

        suggestions
    }

    /// Determine error severity
    fn determine_severity(&self, error: &ProtoHttpParserError) -> ErrorSeverity {
        match error {
            ProtoHttpParserError::Parse(_) => ErrorSeverity::Error,
            ProtoHttpParserError::Validation(_) => ErrorSeverity::Error,
            ProtoHttpParserError::CodeGeneration(_) => ErrorSeverity::Error,
            ProtoHttpParserError::Template(_) => ErrorSeverity::Warning,
            ProtoHttpParserError::Io(_) => ErrorSeverity::Error,
            ProtoHttpParserError::Config(_) => ErrorSeverity::Warning,
            ProtoHttpParserError::Plugin(_) => ErrorSeverity::Warning,
        }
    }

    /// Get error type string
    fn get_error_type(&self, error: &ProtoHttpParserError) -> String {
        match error {
            ProtoHttpParserError::Parse(_) => "Parse".to_string(),
            ProtoHttpParserError::Validation(_) => "Validation".to_string(),
            ProtoHttpParserError::CodeGeneration(_) => "CodeGeneration".to_string(),
            ProtoHttpParserError::Template(_) => "Template".to_string(),
            ProtoHttpParserError::Io(_) => "IO".to_string(),
            ProtoHttpParserError::Config(_) => "Config".to_string(),
            ProtoHttpParserError::Plugin(_) => "Plugin".to_string(),
        }
    }

    /// Get error code for documentation lookup
    fn get_error_code(&self, error: &ProtoHttpParserError) -> Option<String> {
        match error {
            ProtoHttpParserError::Parse(ParseError::Syntax { .. }) => Some("P001".to_string()),
            ProtoHttpParserError::Parse(ParseError::UnexpectedToken { .. }) => Some("P002".to_string()),
            ProtoHttpParserError::Parse(ParseError::ImportNotFound { .. }) => Some("P003".to_string()),
            ProtoHttpParserError::Validation(ValidationError::UndefinedType { .. }) => Some("V001".to_string()),
            ProtoHttpParserError::Validation(ValidationError::DuplicateDefinition { .. }) => Some("V002".to_string()),
            ProtoHttpParserError::Validation(ValidationError::InvalidHttpAnnotation { .. }) => Some("V003".to_string()),
            _ => None,
        }
    }

    /// Format error header
    fn format_error_header(&self, report: &DetailedErrorReport) -> String {
        let severity_str = match report.severity {
            ErrorSeverity::Error => "error",
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Info => "info",
            ErrorSeverity::Hint => "hint",
        };

        let mut header = format!("{}: {}", severity_str, report.error);

        if self.config.show_error_codes {
            if let Some(ref code) = report.error_code {
                header = format!("{} [{}]", header, code);
            }
        }

        if let Some(ref location) = report.location {
            if let Some(ref file) = location.file {
                header = format!("{}\n  --> {}:{}:{}", header, file.display(), location.line, location.column);
            } else {
                header = format!("{}\n  --> line {}:{}", header, location.line, location.column);
            }
        }

        header
    }

    /// Format source context
    fn format_source_context(&self, context: &SourceContext) -> String {
        let mut output = String::new();

        // Before lines
        for line in &context.before_lines {
            output.push_str(&format!("{:4} | {}\n", line.line_number, line.content));
        }

        // Error line
        output.push_str(&format!("{:4} | {}\n", context.error_line.line_number, context.error_line.content));

        // Error indicator
        if let Some((start, end)) = context.error_range {
            let spaces = " ".repeat(7 + start); // 4 digits + " | " + start columns
            let carets = "^".repeat(end - start);
            output.push_str(&format!("{}{}\n", spaces, carets));
        }

        // After lines
        for line in &context.after_lines {
            output.push_str(&format!("{:4} | {}\n", line.line_number, line.content));
        }

        output
    }

    /// Format error details
    fn format_error_details(&self, report: &DetailedErrorReport) -> String {
        format!("Error: {}", report.error)
    }

    /// Format suggestions
    fn format_suggestions(&self, suggestions: &[ValidationSuggestion]) -> String {
        let mut output = String::new();
        output.push_str("Suggestions:\n");

        for (i, suggestion) in suggestions.iter().enumerate() {
            output.push_str(&format!("  {}: {}", i + 1, suggestion.message));
            if let Some(ref fix) = suggestion.suggested_fix {
                output.push_str(&format!(" (try: '{}')", fix));
            }
            output.push('\n');
        }

        output
    }

    /// Format error summary
    fn format_summary(&self, stats: &ErrorStats) -> String {
        format!("\nSummary: {} error(s) in {} file(s)", 
                stats.total_errors, 
                stats.files_with_errors.len())
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_reporter_creation() {
        let reporter = ErrorReporter::new();
        assert_eq!(reporter.config.context_lines, 2);
        assert!(reporter.config.show_suggestions);
    }

    #[test]
    fn test_error_severity_determination() {
        let reporter = ErrorReporter::new();
        
        let parse_error = ProtoHttpParserError::Parse(ParseError::Syntax {
            line: 1,
            column: 1,
            message: "test".to_string(),
        });
        
        assert_eq!(reporter.determine_severity(&parse_error), ErrorSeverity::Error);
    }

    #[test]
    fn test_error_code_generation() {
        let reporter = ErrorReporter::new();
        
        let syntax_error = ProtoHttpParserError::Parse(ParseError::Syntax {
            line: 1,
            column: 1,
            message: "test".to_string(),
        });
        
        assert_eq!(reporter.get_error_code(&syntax_error), Some("P001".to_string()));
    }
}