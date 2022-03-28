use std::collections::HashMap;

use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind, ParseError},
    grapheme::Grapheme,
    group::Group,
    literal::Literal,
    lookaround::Lookaround,
    options::{CompileOptions, ParseOptions},
    range::Range,
    reference::Reference,
    repetition::{RegexQuantifier, Repetition},
    span::Span,
    stmt::StmtExpr,
    var::Variable,
};

/// A parsed rulex expression, which might contain more sub-expressions.
#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rulex<'i> {
    /// A string literal
    Literal(Literal<'i>),
    /// A character class
    CharClass(CharClass),
    /// A Unicode grapheme
    Grapheme(Grapheme),
    /// A group, i.e. a sequence of rules, possibly wrapped in parentheses.
    Group(Group<'i>),
    /// An alternation, i.e. a list of alternatives; at least one of them has to match.
    Alternation(Alternation<'i>),
    /// A repetition, i.e. a expression that must be repeated. The number of required repetitions is
    /// constrained by a lower and possibly an upper bound.
    Repetition(Box<Repetition<'i>>),
    /// A boundary (start of string, end of string or word boundary).
    Boundary(Boundary),
    /// A (positive or negative) lookahead or lookbehind.
    Lookaround(Box<Lookaround<'i>>),
    /// An variable that has been declared before.
    Variable(Variable<'i>),
    /// A backreference or forward reference.
    Reference(Reference<'i>),
    /// A range of integers
    Range(Range),
    /// An expression preceded by a modifier such as `enable lazy;`
    StmtExpr(Box<StmtExpr<'i>>),
}

impl<'i> Rulex<'i> {
    pub fn parse(input: &'i str, _options: ParseOptions) -> Result<Self, ParseError> {
        crate::parse::parse(input)
    }

    pub fn compile(&self, options: CompileOptions) -> Result<String, CompileError> {
        let mut used_names = HashMap::new();
        let mut groups_count = 0;
        self.get_capturing_groups(&mut groups_count, &mut used_names, false)?;

        let mut state = CompileState {
            next_idx: 1,
            used_names,
            groups_count,
            default_quantifier: RegexQuantifier::Greedy,
            variables: vec![],
            current_vars: Default::default(),
        };
        let compiled = self.comp(options, &mut state)?;

        let mut buf = String::new();
        compiled.codegen(&mut buf, options.flavor);
        Ok(buf)
    }

    pub fn parse_and_compile(input: &str, options: CompileOptions) -> Result<String, CompileError> {
        let parsed = Rulex::parse(input, options.parse_options)?;
        parsed.compile(options)
    }

    pub(crate) fn span(&self) -> Span {
        match self {
            Rulex::Literal(l) => l.span,
            Rulex::CharClass(c) => c.span,
            Rulex::Grapheme(g) => g.span,
            Rulex::Group(g) => g.span,
            Rulex::Alternation(a) => a.span,
            Rulex::Repetition(r) => r.span,
            Rulex::Boundary(b) => b.span,
            Rulex::Lookaround(l) => l.span,
            Rulex::Variable(v) => v.span,
            Rulex::Reference(r) => r.span,
            Rulex::Range(r) => r.span,
            Rulex::StmtExpr(m) => m.span,
        }
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self {
            Rulex::Literal(_) => {}
            Rulex::CharClass(_) => {}
            Rulex::Grapheme(_) => {}
            Rulex::Group(g) => g.get_capturing_groups(count, map, within_variable)?,
            Rulex::Alternation(a) => a.get_capturing_groups(count, map, within_variable)?,
            Rulex::Repetition(r) => r.get_capturing_groups(count, map, within_variable)?,
            Rulex::Boundary(_) => {}
            Rulex::Lookaround(l) => l.get_capturing_groups(count, map, within_variable)?,
            Rulex::Variable(_) => {}
            Rulex::Reference(r) => {
                if within_variable {
                    return Err(CompileErrorKind::ReferenceInLet.at(r.span));
                }
            }
            Rulex::Range(_) => {}
            Rulex::StmtExpr(m) => m.get_capturing_groups(count, map, within_variable)?,
        }
        Ok(())
    }

    pub(crate) fn comp<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match self {
            Rulex::Literal(l) => l.compile(),
            Rulex::CharClass(c) => c.compile(options),
            Rulex::Group(g) => g.compile(options, state),
            Rulex::Grapheme(g) => g.compile(options),
            Rulex::Alternation(a) => a.compile(options, state),
            Rulex::Repetition(r) => r.compile(options, state),
            Rulex::Boundary(b) => b.compile(),
            Rulex::Lookaround(l) => l.compile(options, state),
            Rulex::Variable(v) => v.compile(options, state),
            Rulex::Reference(r) => r.compile(options, state),
            Rulex::Range(r) => r.compile(),
            Rulex::StmtExpr(m) => m.compile(options, state),
        }
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Rulex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Rulex::Literal(arg0) => arg0.fmt(f),
            Rulex::CharClass(arg0) => arg0.fmt(f),
            Rulex::Grapheme(arg0) => arg0.fmt(f),
            Rulex::Group(arg0) => arg0.fmt(f),
            Rulex::Alternation(arg0) => arg0.fmt(f),
            Rulex::Repetition(arg0) => arg0.fmt(f),
            Rulex::Boundary(arg0) => arg0.fmt(f),
            Rulex::Lookaround(arg0) => arg0.fmt(f),
            Rulex::Variable(arg0) => arg0.fmt(f),
            Rulex::Reference(arg0) => arg0.fmt(f),
            Rulex::Range(arg0) => arg0.fmt(f),
            Rulex::StmtExpr(arg0) => arg0.fmt(f),
        }
    }
}
