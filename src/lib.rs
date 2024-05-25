use chumsky::prelude::*;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};
use wasmtime::{Engine, Instance, Module as WasmtimeModule, Store};

use anyhow::{Context, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(f64),
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
            .map(|(l, r)| Number(format!("{}.{}", l, r).parse::<f64>().unwrap()))
            .or(num.map(|s: &str| Number(s.parse::<f64>().unwrap())));

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

// TODO: Use Spans for error handling & reporting during evaluation
const EPSILON: f64 = 0.00001;
const ENTRYPOINT: &str = "run";

/// calculate the numerical result of an expression
pub fn evaluate(expr: Expr) -> f64 {
    match expr {
        Expr::Number(num) => num,
        Expr::Negated(expr) => -evaluate(*expr),
        Expr::Reciprocal(expr) => 1.0 / (evaluate(*expr) + EPSILON), // TODO: Raise a divide by zero error during parsing
        Expr::Add(left, right) => evaluate(*left) + evaluate(*right),
        Expr::Multiply(left, right) => evaluate(*left) * evaluate(*right),
    }
}

/// calculate the numerical result of an expression via JIT compilation to Web Assembly. The generated Web Assembly module is executed via `wasmtime`.
pub fn evaluate_jit(expr: Expr) -> Result<f64> {
    let compiled = compile(expr);

    let engine = Engine::default();
    let module = WasmtimeModule::new(&engine, &compiled)
        .context("Failed to load compiled expression into wasmtime")?;

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])
        .context("Failed to instansiate compiled expression")?;

    let run = instance
        .get_func(&mut store, ENTRYPOINT)
        .context("Failed to find `run` function in compiled expression")?;

    let answer = run
        .typed::<(), f64>(&store)
        .context("Failed to type `run` function in compiled expression")?;

    let answer = answer
        .call(&mut store, ())
        .context("Failed to execute `run` function in compiled expression")?;

    Ok(answer)
}

fn compile(expr: Expr) -> Vec<u8> {
    let mut module = Module::new();

    // type section
    let mut types = TypeSection::new();
    types.function(vec![], vec![ValType::F64]);
    module.section(&types);

    // function section
    let mut functions = FunctionSection::new();
    functions.function(0); // function type index is 0 since it's the only type we've defined
    module.section(&functions);

    // export section
    let mut exports = ExportSection::new();
    exports.export(ENTRYPOINT, ExportKind::Func, 0); // function index is 0, same logic
    module.section(&exports);

    // code section
    let mut codes = CodeSection::new();
    let mut run = Function::new(vec![]);

    codegen(&mut run, expr);
    run.instruction(&Instruction::End);

    codes.function(&run);
    module.section(&codes);

    let bytes = module.finish();
    bytes
}

fn codegen(func: &mut Function, expr: Expr) {
    match expr {
        Expr::Number(num) => {
            func.instruction(&Instruction::F64Const(num));
        }
        Expr::Negated(expr) => {
            codegen(func, *expr);
            func.instruction(&Instruction::F64Neg);
        }
        Expr::Reciprocal(expr) => {
            func.instruction(&Instruction::F64Const(1.0));
            codegen(func, *expr);

            // TODO: instead of this, raise a divide by zero error
            // add an epsilon
            func.instruction(&Instruction::F64Const(EPSILON));
            func.instruction(&Instruction::F64Add);

            func.instruction(&Instruction::F64Div);
        }
        Expr::Add(left, right) => {
            codegen(func, *left);
            codegen(func, *right);
            func.instruction(&Instruction::F64Add);
        }
        Expr::Multiply(left, right) => {
            codegen(func, *left);
            codegen(func, *right);
            func.instruction(&Instruction::F64Mul);
        }
    }
}
