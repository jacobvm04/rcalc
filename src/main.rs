use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Integer(f32),
    Negated(Box<Expr>),
    Reciprocal(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
}

fn parser<'src>(
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
        Expr::Reciprocal(expr) => 1.0 / (evaluate(*expr) + 0.001),
        Expr::Add(left, right) => evaluate(*left) + evaluate(*right),
        Expr::Multiply(left, right) => evaluate(*left) * evaluate(*right),
    }
}

fn main() {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join(" ");

    let (expr, errs) = parser().parse(&input).into_output_errors();

    if errs.len() > 0 {
        println!("Failed to parse: {:?}", errs);
    } else {
        let result = evaluate(expr.unwrap());
        println!("{}", result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn expr_to_string(expr: Expr) -> String {
        match expr {
            Expr::Integer(num) => num.to_string(),
            Expr::Negated(expr) => format!("-({})", evaluate(*expr)),
            Expr::Reciprocal(expr) => format!("(1.0 / {})", evaluate(*expr)),
            Expr::Add(left, right) => format!("({} + {})", evaluate(*left), evaluate(*right)),
            Expr::Multiply(left, right) => format!("({} * {})", evaluate(*left), evaluate(*right)),
        }
    }

    fn arb_expr() -> impl Strategy<Value = Expr> {
        use Expr::*;

        // prop::arbitrary::any::<f32>() doesn't pass yet since scientific notation parsing is not implemented
        // let leaf = prop_oneof![prop::arbitrary::any::<f32>().prop_map(Integer),];

        let leaf = prop_oneof![(-100000000f32..100000000f32).prop_map(Integer),];

        leaf.prop_recursive(8, 256, 10, |inner| {
            prop_oneof![
                inner.clone().prop_map(|expr| Negated(expr.into())),
                inner.clone().prop_map(|expr| Reciprocal(expr.into())),
                (inner.clone(), inner.clone())
                    .prop_map(|(left, right)| Add(left.into(), right.into())),
                (inner.clone(), inner.clone())
                    .prop_map(|(left, right)| Multiply(left.into(), right.into())),
            ]
        })
    }

    proptest! {
        #[test]
        fn test_parse_arbitrary_expr(expr in arb_expr()) {
            let expr_string = expr_to_string(expr.clone());
            let (new_expr, _) = parser().parse(&expr_string).into_output_errors();

            prop_assert_eq!(evaluate(new_expr.unwrap()), evaluate(expr));
        }

        #[test]
        fn test_parse_arbitrary_expr_cli(expr in arb_expr()) {
            let expr_string = expr_to_string(expr.clone());

            let mut cmd = assert_cmd::Command::cargo_bin("rcalc").unwrap();

            cmd.arg(expr_string);
            cmd.assert().success().stdout(predicates::function::function(|output: &str| output.trim().parse::<f32>().unwrap() == evaluate(expr.clone())));
        }
    }
}
