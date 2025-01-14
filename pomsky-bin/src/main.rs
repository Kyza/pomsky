use std::{io, io::Write as _, process::exit, time::Instant};

use pomsky::{
    diagnose::{Diagnostic, Severity},
    options::{CompileOptions, RegexFlavor},
    Expr,
};

#[macro_use]
mod format;
mod args;
mod result;

use args::{Args, DiagnosticSet, Input};
use result::CompilationResult;

pub fn main() {
    let args = match args::parse_args() {
        Ok(args) => args,
        Err(error) => {
            print_diagnostic(
                &Diagnostic::ad_hoc(Severity::Error, None, error.to_string(), None),
                None,
            );
            args::print_short_usage_and_help_err();
            exit(2)
        }
    };

    match &args.input {
        Input::Value(input) => compile(input, &args),
        Input::File(path) => match std::fs::read_to_string(path) {
            Ok(input) => compile(&input, &args),
            Err(error) => {
                print_diagnostic(
                    &Diagnostic::ad_hoc(Severity::Error, None, error.to_string(), None),
                    None,
                );
                exit(3);
            }
        },
    }
}

fn compile(input: &str, args: &Args) {
    let start = Instant::now();

    let options = CompileOptions {
        flavor: args.flavor.unwrap_or(RegexFlavor::Pcre),
        max_range_size: 12,
        allowed_features: args.allowed_features,
    };

    let (parsed, warnings) = match Expr::parse(input) {
        (Some(res), warnings) => (res, warnings),
        (None, err) => {
            print_parse_errors(err, Some(input), start.elapsed().as_micros(), args.json);
            exit(1);
        }
    };
    let mut warnings = warnings.collect::<Vec<_>>();

    if args.debug {
        eprintln!("======================== debug ========================");
        eprintln!("{parsed:#?}\n");
    }

    if !args.json {
        print_warnings(&warnings, args, Some(input));
    }

    let compiled = match parsed.compile(input, options) {
        (Some(res), compile_warnings) => {
            if args.json {
                warnings.extend(compile_warnings);
            } else {
                print_warnings(&compile_warnings, args, Some(input));
            }

            res
        }
        (None, errors) => {
            if args.json {
                CompilationResult::error(start.elapsed().as_micros())
                    .with_diagnostics(errors, Some(input))
                    .with_diagnostics(
                        warnings.into_iter().filter_map(|w| {
                            if args.warnings.is_enabled(w.kind) {
                                Some(w)
                            } else {
                                None
                            }
                        }),
                        Some(input),
                    )
                    .output_json();
            } else {
                for err in &errors {
                    print_diagnostic(err, Some(input));
                }
            }
            std::process::exit(1);
        }
    };

    if args.json {
        CompilationResult::success(compiled, start.elapsed().as_micros())
            .with_diagnostics(
                warnings.into_iter().filter_map(|w| {
                    if args.warnings.is_enabled(w.kind) {
                        Some(w)
                    } else {
                        None
                    }
                }),
                Some(input),
            )
            .output_json();
    } else if args.no_new_line {
        print!("{compiled}");
        io::stdout().flush().unwrap();
    } else {
        println!("{compiled}");
    }
}

fn print_parse_errors(
    mut diagnostics: impl Iterator<Item = Diagnostic>,
    source_code: Option<&str>,
    time: u128,
    json: bool,
) {
    if json {
        CompilationResult::error(time).with_diagnostics(diagnostics, source_code).output_json();
    } else {
        let mut len = 0;
        for d in (&mut diagnostics).take(8) {
            len += 1;
            print_diagnostic(&d, source_code);
        }

        len += diagnostics.count();

        if len > 8 {
            efprintln!(C!"note" ": some errors were omitted");
        }

        if len > 1 {
            let len = &len.to_string();
            efprintln!(R!"error" ": could not compile expression due to " {len} " previous errors");
        } else {
            efprintln!(R!"error" ": could not compile expression due to previous error");
        }
    }
}

fn print_warnings(warnings: &[Diagnostic], args: &Args, source_code: Option<&str>) {
    if matches!(&args.warnings, DiagnosticSet::Enabled(set) if set.is_empty()) {
        return;
    }

    let mut len = 0;

    for diagnostic in warnings {
        if args.warnings.is_enabled(diagnostic.kind) {
            len += 1;
            match len {
                1..=8 => print_diagnostic(diagnostic, source_code),
                9 => efprintln!(C!"note" ": some warnings were omitted"),
                _ => {}
            }
        }
    }

    if len > 1 {
        let len = len.to_string();
        efprintln!(Y!"warning" ": pomsky generated " {&len} " warnings");
    }
}

fn print_diagnostic(diagnostic: &Diagnostic, source_code: Option<&str>) {
    let kind = diagnostic.kind.to_string();
    let display = diagnostic.default_display(source_code).to_string();
    if let Some(code) = diagnostic.code {
        let code = code.to_string();
        match diagnostic.severity {
            Severity::Error => efprintln!(R!"error " R!{&code} {&kind} ": " {&display}),
            Severity::Warning => efprintln!(Y!"warning " Y!{&code} {&kind} ": " {&display}),
        }
    } else {
        match diagnostic.severity {
            Severity::Error => efprintln!(R!"error" {&kind} ": " {&display}),
            Severity::Warning => efprintln!(Y!"warning" {&kind} ": " {&display}),
        }
    }
}
