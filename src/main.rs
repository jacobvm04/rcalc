#![feature(unix_sigpipe)]

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;

#[unix_sigpipe = "sig_dfl"] // needed for unix pipes to work correctly
fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let (expr, errs) = rcalc::parser().parse(&input).into_output_errors();

    if errs.len() > 0 {
        eprintln!("Failed to parse input expression");

        errs.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new(e.span().into_range())
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(&input))
                .unwrap()
        });
    } else {
        let result = rcalc::evaluate(expr.unwrap());
        println!("{}", result);
    }
}
