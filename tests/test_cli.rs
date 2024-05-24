use chumsky::prelude::*;
use proptest::prelude::*;
use rcalc::*;

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
            (inner.clone(), inner.clone()).prop_map(|(left, right)| Add(left.into(), right.into())),
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
