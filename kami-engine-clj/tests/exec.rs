//! Execution-grade compiler tests — compile an expression, RUN it, assert the value.
//!
//! Unlike `basic.rs` (which only checks that `\0asm` bytes came out), these run the
//! compiled module under wasmtime and assert what it actually computes. That is the
//! only kind of test that catches silent codegen bugs. Requires `--features run`.
#![cfg(feature = "run")]

use kami_engine_clj::run::eval_i64;

fn eval(expr: &str) -> i64 {
    eval_i64(expr).unwrap_or_else(|e| panic!("eval `{expr}` failed: {e:?}"))
}

#[test]
fn arithmetic_computes_correct_values() {
    assert_eq!(eval("(+ 2 3)"), 5);
    assert_eq!(eval("(+ 1 2 3 4)"), 10); // variadic add
    assert_eq!(eval("(- 10 3)"), 7);
    assert_eq!(eval("(- 10 3 2)"), 5); // variadic sub
    assert_eq!(eval("(* 4 5)"), 20);
    assert_eq!(eval("(* 2 3 4)"), 24); // variadic mul
    assert_eq!(eval("(quot 17 5)"), 3);
    assert_eq!(eval("(mod 17 5)"), 2);
    assert_eq!(eval("(inc 41)"), 42);
    assert_eq!(eval("(dec 1)"), 0);
}

#[test]
fn two_arg_comparisons_are_correct() {
    assert_eq!(eval("(= 3 3)"), 1);
    assert_eq!(eval("(= 3 4)"), 0);
    assert_eq!(eval("(< 1 2)"), 1);
    assert_eq!(eval("(< 2 1)"), 0);
    assert_eq!(eval("(> 5 2)"), 1);
    assert_eq!(eval("(<= 2 2)"), 1);
    assert_eq!(eval("(>= 2 3)"), 0);
}

/// REGRESSION GUARD: multi-arg `=` must mean "all equal", not fold the boolean
/// result back into the next comparison. `(= 5 5 5)` was returning 0 before the fix
/// (push 5; 5==5→1; then 1==5→0) — a silent unsoundness any chained equality hit.
#[test]
fn multi_arg_equality_means_all_equal() {
    assert_eq!(eval("(= 1 1 1)"), 1);
    assert_eq!(eval("(= 5 5 5)"), 1); // was 0 — the bug
    assert_eq!(eval("(= 7 7 7 7)"), 1);
    assert_eq!(eval("(= 5 5 6)"), 0);
    assert_eq!(eval("(= 1 2 1)"), 0);
}

/// REGRESSION GUARD: ordered comparisons with >2 args must check EVERY adjacent
/// pair. The old codegen only compared args[0] and args[1] and silently dropped the
/// rest — `(< 1 2 0)` returned 1 (true) when 2 < 0 is false.
#[test]
fn multi_arg_ordering_checks_every_pair() {
    assert_eq!(eval("(< 1 2 3)"), 1);
    assert_eq!(eval("(< 1 2 0)"), 0); // was 1 — the dropped-tail bug
    assert_eq!(eval("(> 3 2 1)"), 1);
    assert_eq!(eval("(> 3 2 5)"), 0);
    assert_eq!(eval("(<= 1 1 2)"), 1);
    assert_eq!(eval("(<= 1 2 2 1)"), 0);
    assert_eq!(eval("(>= 5 5 1)"), 1);
}

#[test]
fn conditionals_pick_the_right_branch() {
    assert_eq!(eval("(if (< 1 2) 100 200)"), 100);
    assert_eq!(eval("(if (< 2 1) 100 200)"), 200);
    assert_eq!(eval("(if (= 3 3 3) 1 0)"), 1);
    assert_eq!(eval("(let [a 10 b 20] (+ a b))"), 30);
}
