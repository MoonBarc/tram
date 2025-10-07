use std::{borrow::Cow, ops::Range};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
        }
    }

    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn exact_range(&self, source: &str) -> Range<usize> {
        let end = self.end.min(source.len() - 1);
        self.start.min(end - 1)..end
    }

    pub fn surrounding_range(&self, source: &str) -> Range<usize> {
        self.start.saturating_sub(10) .. (self.end + 10).min(source.len() - 1)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct ParseError {
    pub span: Span,
    pub message: Cow<'static, str>
}

impl ParseError {
    pub fn log(&self, source: Option<&str>) {
        eprintln!("== Parse Error: {}", self.message);
        if let Some(source) = source {
            eprintln!("problem at:");
            let surrounding = self.span.surrounding_range(source);
            let exact = self.span.exact_range(source);
            eprintln!(
                ">| {}\x1b[31m\x1b[4:3m{}\x1b[0m{}",
                &source[surrounding.start .. exact.start],
                &source[exact.clone()],
                &source[exact.end .. surrounding.end]
            );
        }
    }
}
