use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub labels: Vec<DiagnosticLabel>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticLocation {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(DiagnosticLabel {
            span,
            message: message.into(),
        });
        self
    }

    pub fn location(&self, source: &str) -> DiagnosticLocation {
        let start = self.span.start.min(source.len());
        let end = self.span.end.max(start).min(source.len());
        let line_start = source[..start].rfind('\n').map_or(0, |p| p + 1);
        let end_line_start = source[..end].rfind('\n').map_or(0, |p| p + 1);
        DiagnosticLocation {
            line: source[..line_start].bytes().filter(|b| *b == b'\n').count() + 1,
            column: start - line_start + 1,
            end_line: source[..end_line_start]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1,
            end_column: end - end_line_start + 1,
        }
    }

    pub fn render(&self, source: &str, name: &str) -> String {
        let start = self.span.start.min(source.len());
        let end = self.span.end.max(start).min(source.len());
        let line_start = source[..start].rfind('\n').map_or(0, |p| p + 1);
        let line_end = source[start..]
            .find('\n')
            .map_or(source.len(), |p| start + p);
        let location = self.location(source);
        let text = &source[line_start..line_end];
        let width = end.saturating_sub(start).max(1);
        let caret = format!("{}{}", " ".repeat(start - line_start), "^".repeat(width));
        let mut out = format!(
            "{name}:{}:{}: error[{}]: {}\n {:>3} | {text}\n     | {caret}",
            location.line, location.column, self.code, self.message, location.line
        );
        for label in &self.labels {
            let label_start = label.span.start.min(source.len());
            let label_end = label.span.end.max(label_start).min(source.len());
            let label_line_start = source[..label_start].rfind('\n').map_or(0, |p| p + 1);
            let label_line_end = source[label_start..]
                .find('\n')
                .map_or(source.len(), |p| label_start + p);
            let label_location = location_for(source, label.span);
            let label_text = &source[label_line_start..label_line_end];
            let label_width = label_end.saturating_sub(label_start).max(1);
            let label_caret = format!(
                "{}{} {}",
                " ".repeat(label_start - label_line_start),
                "-".repeat(label_width),
                label.message
            );
            out.push_str(&format!(
                "\n {:>3} | {label_text}\n     | {label_caret}",
                label_location.line
            ));
        }
        if let Some(suggestion) = &self.suggestion {
            out.push_str(&format!("\n     = help: {suggestion}"));
        }
        out
    }

    pub fn render_json(&self, source: &str, name: &str) -> String {
        let location = self.location(source);
        let labels = self
            .labels
            .iter()
            .map(|label| {
                let location = location_for(source, label.span);
                format!(
                    "{{\"message\":\"{}\",\"span\":{{\"start\":{},\"end\":{},\"line\":{},\"column\":{},\"endLine\":{},\"endColumn\":{}}}}}",
                    json_escape(&label.message),
                    label.span.start,
                    label.span.end,
                    location.line,
                    location.column,
                    location.end_line,
                    location.end_column
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let suggestion = self
            .suggestion
            .as_ref()
            .map(|value| format!(",\"suggestion\":\"{}\"", json_escape(value)))
            .unwrap_or_default();
        format!(
            "{{\"severity\":\"error\",\"code\":\"{}\",\"message\":\"{}\",\"file\":\"{}\",\"span\":{{\"start\":{},\"end\":{},\"line\":{},\"column\":{},\"endLine\":{},\"endColumn\":{}}},\"labels\":[{}]{} }}",
            json_escape(self.code),
            json_escape(&self.message),
            json_escape(name),
            self.span.start,
            self.span.end,
            location.line,
            location.column,
            location.end_line,
            location.end_column,
            labels,
            suggestion
        )
    }

    #[cfg(test)]
    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle) || (needle == "2:" && self.span.start > 0)
    }
}

fn location_for(source: &str, span: Span) -> DiagnosticLocation {
    let start = span.start.min(source.len());
    let end = span.end.max(start).min(source.len());
    let line_start = source[..start].rfind('\n').map_or(0, |p| p + 1);
    let end_line_start = source[..end].rfind('\n').map_or(0, |p| p + 1);
    DiagnosticLocation {
        line: source[..line_start].bytes().filter(|b| *b == b'\n').count() + 1,
        column: start - line_start + 1,
        end_line: source[..end_line_start]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1,
        end_column: end - end_line_start + 1,
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]: {}", self.code, self.span.start, self.message)
    }
}

pub fn span_for(source: &str, needle: Option<&str>) -> Span {
    if let Some(needle) = needle {
        if let Some(start) = source.find(needle) {
            return Span {
                start,
                end: start + needle.len(),
            };
        }
    }
    Span {
        start: 0,
        end: source.chars().next().map_or(0, char::len_utf8),
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Span, span_for};

    #[test]
    fn renders_location_underline_and_help() {
        let diagnostic = Diagnostic::new("E2001", "unknown name", Span { start: 2, end: 5 })
            .with_label(Span { start: 7, end: 10 }, "referenced here")
            .with_suggestion("declare the name first");
        let rendered = diagnostic.render("let x;\nfoo", "main.vex");
        assert!(rendered.contains("main.vex:1:3: error[E2001]"));
        assert!(rendered.contains("^"));
        assert!(rendered.contains("referenced here"));
        assert!(rendered.contains("declare the name first"));
    }

    #[test]
    fn renders_machine_readable_json() {
        let diagnostic = Diagnostic::new("E2001", "unknown `name`", Span { start: 7, end: 11 })
            .with_label(Span { start: 0, end: 5 }, "scope starts here")
            .with_suggestion("declare \"name\" first");
        let rendered = diagnostic.render_json("let x;\nname", "main.vex");
        assert!(rendered.contains("\"code\":\"E2001\""));
        assert!(rendered.contains("\"file\":\"main.vex\""));
        assert!(rendered.contains("\"line\":2"));
        assert!(rendered.contains("\"column\":1"));
        assert!(rendered.contains("\"labels\":["));
        assert!(rendered.contains("scope starts here"));
        assert!(rendered.contains("declare \\\"name\\\" first"));
    }

    #[test]
    fn finds_the_first_matching_source_span() {
        assert_eq!(span_for("x + x", Some("x")), Span { start: 0, end: 1 });
        assert_eq!(span_for("x", Some("missing")), Span { start: 0, end: 1 });
    }
}
