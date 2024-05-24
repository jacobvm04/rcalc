# RCalc

`rcalc` is a simple CLI calculator for basic floating point expressions. I built this project as a way to learn about writing [parser cominbators](https://en.wikipedia.org/wiki/Parser_combinator) in rust using `chumsky`.

Usage

```sh
> rcalc '1 + 1 * 2 / 4'
1.4998751
```

Malformed input example

```sh
❯ rcalc 2 -+3
Failed to parse input expression
Error: found + expected '(', or '-'
   ╭─[<unknown>:1:4]
   │
 1 │ 2 -+3
   │    ┬
   │    ╰── found + expected '(', or '-'
```

## Installation

To install `rcalc`, clone this repo and run `cargo install --path .` in the cloned directory. Do note that this project requires a `nightly` copy of rust to build.

## Tests

This project uses `proptest` for property based testing. To run the tests, simply run

```sh
> cargo test # or `cargo nextest run` if you have `cargo-nextest` installed
```

For code coverage, install `cargo-llvm-cov` and run

```sh
cargo llvm-cov
```
