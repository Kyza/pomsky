//! Provides warnings that are shown to the user (in addition to the output)

use std::fmt;

use crate::span::Span;

/// A warning.
#[derive(Clone, Copy)]
pub struct ParseWarning {
    /// The kind of warning
    pub kind: ParseWarningKind,
    /// The span pointing to the source of the warning
    pub span: Span,
}

/// A warning without a span pointing to the source of the warning
#[derive(Debug, Clone, Copy)]
pub enum ParseWarningKind {
    /// A deprecation warning
    Deprecation(DeprecationWarning),
}

impl ParseWarningKind {
    pub(crate) fn at(self, span: Span) -> ParseWarning {
        ParseWarning { kind: self, span }
    }
}

impl fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseWarningKind::Deprecation(warning) = &self.kind;
        if let Some(std::ops::Range { start, end }) = self.span.range() {
            write!(f, "{warning}\n  at {start}..{end}")
        } else {
            write!(f, "{warning}")
        }
    }
}

impl fmt::Display for ParseWarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ParseWarningKind::Deprecation(c) = self;
        c.fmt(f)
    }
}

/// A deprecation warning: Indicates that something shouldn't be used anymore
#[derive(Debug, Clone, Copy)]
pub enum DeprecationWarning {
    /// The `[.]` dot
    Dot,
}

impl fmt::Display for DeprecationWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeprecationWarning::Dot => {
                f.write_str("This syntax is deprecated. Use `.` without the brackets.")
            }
        }
    }
}
