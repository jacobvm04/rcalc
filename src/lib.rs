use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Integer(f32),
    Negated(Box<Expr>),
    Reciprocal(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
}

pub fn parser<'src>(
) -> impl Parser<'src, &'src str, Expr, extra::Err<Rich<'src, char, SimpleSpan<usize>>>> {
    use Expr::*;

    recursive(|expr| {
        let num = text::digits(10).to_slice();

        let decimal = num
            .then(just(".").ignore_then(num))
            .map(|(l, r)| Integer(format!("{}.{}", l, r).parse::<f32>().unwrap()))
            .or(num.map(|s: &str| Integer(s.parse::<f32>().unwrap())));

        let atom = decimal.or(expr.delimited_by(just("("), just(")"))).padded();

        let unary = atom
            .clone()
            .or(just("-").ignore_then(atom).map(|a| Negated(Box::new(a))))
            .padded();

        let product = unary.clone().foldl(
            just("*")
                .or(just("x"))
                .or(just("/"))
                .then(unary.clone())
                .repeated(),
            |l, (op, r)| match op {
                "*" | "x" => Multiply(Box::new(l), Box::new(r)),
                "/" => Multiply(Box::new(l), Box::new(Reciprocal(Box::new(r)))),
                _ => unreachable!(),
            },
        );

        product.clone().foldl(
            just("+").or(just("-")).then(product).repeated(),
            |l, (op, r)| match op {
                "+" => Add(Box::new(l), Box::new(r)),
                "-" => Add(Box::new(l), Box::new(Negated(Box::new(r)))),
                _ => unreachable!(),
            },
        )
    })
}

pub fn evaluate(expr: Expr) -> f32 {
    match expr {
        Expr::Integer(num) => num,
        Expr::Negated(expr) => -evaluate(*expr),
        Expr::Reciprocal(expr) => 1.0 / (evaluate(*expr) + 0.00001), // TODO: Raise a divide by zero error during parsing
        Expr::Add(left, right) => evaluate(*left) + evaluate(*right),
        Expr::Multiply(left, right) => evaluate(*left) * evaluate(*right),
    }
}
