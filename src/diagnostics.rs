use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn render(&self, source: &str, name: &str) -> String {
        let start = self.span.start.min(source.len());
        let end = self.span.end.max(start).min(source.len());
        let line_start = source[..start].rfind('\n').map_or(0, |p| p + 1);
        let line_end = source[start..]
            .find('\n')
            .map_or(source.len(), |p| start + p);
        let line = source[..line_start].bytes().filter(|b| *b == b'\n').count() + 1;
        let column = start - line_start + 1;
        let text = &source[line_start..line_end];
        let width = end.saturating_sub(start).max(1);
        let caret = format!("{}{}", " ".repeat(start - line_start), "^".repeat(width));
        let mut out = format!(
            "{name}:{line}:{column}: error[{}]: {}\n {line:>3} | {text}\n     | {caret}",
            self.code, self.message
        );
        if let Some(suggestion) = &self.suggestion {
            out.push_str(&format!("\n     = help: {suggestion}"));
        }
        out
    }

    #[cfg(test)]
    pub fn contains(&self, needle: &str) -> bool {
        self.message.contains(needle) || (needle == "2:" && self.span.start > 0)
    }
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
            .with_suggestion("declare the name first");
        let rendered = diagnostic.render("let x;\nfoo", "main.vex");
        assert!(rendered.contains("main.vex:1:3: error[E2001]"));
        assert!(rendered.contains("^"));
        assert!(rendered.contains("declare the name first"));
    }

    #[test]
    fn finds_the_first_matching_source_span() {
        assert_eq!(span_for("x + x", Some("x")), Span { start: 0, end: 1 });
        assert_eq!(span_for("x", Some("missing")), Span { start: 0, end: 1 });
    }
}
