extern crate meval;

use std::{f64::EPSILON, str::FromStr};

use RSet;

lazy_static!(
    static ref RNAMES: Vec<String> = (1..=100).map(|i| format!("R{}", i)).collect();
);

enum Bounds {
    Cmp(Box<Fn(f64, f64) -> bool>, meval::Expr, f64),
    Err(meval::Expr, f64),
}

fn split_expr(expr: &str, pat: &str) -> (meval::Expr, f64) {
    let mut split = expr.split(pat);
    (
        split.next().unwrap().trim().parse::<meval::Expr>().unwrap(),
        split.next().unwrap().trim().parse::<f64>().unwrap(),
    )
}

impl FromStr for Bounds {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.contains("<=") {
            let (ex, trg) = split_expr(s, "<=");
            Ok(Bounds::Cmp(Box::new(|a, b| a <= b), ex, trg))
        } else if s.contains('<') {
            let (ex, trg) = split_expr(s, "<");
            Ok(Bounds::Cmp(Box::new(|a, b| a < b), ex, trg))
        } else if s.contains(">=") {
            let (ex, trg) = split_expr(s, ">=");
            Ok(Bounds::Cmp(Box::new(|a, b| a >= b), ex, trg))
        } else if s.contains('>') {
            let (ex, trg) = split_expr(s, ">");
            Ok(Bounds::Cmp(Box::new(|a, b| a > b), ex, trg))
        } else if s.contains("==") {
            let (ex, trg) = split_expr(s, "==");
            Ok(Bounds::Cmp(
                Box::new(|a, b| (a - b).abs() < EPSILON),
                ex,
                trg,
            ))
        } else if s.contains("!=") {
            let (ex, trg) = split_expr(s, "!=");
            Ok(Bounds::Cmp(
                Box::new(|a, b| (a - b).abs() > EPSILON),
                ex,
                trg,
            ))
        } else if s.contains('~') {
            let (ex, trg) = split_expr(s, "~");
            Ok(Bounds::Err(ex, trg))
        } else {
            Err("Err: Bound must contain either <, <=, >, >=, ==, != or ~")
        }
    }
}

/// Builder struct used to create `f` values for `RCalc::calc` from mathematical expressions.
#[derive(Default)]
pub struct ROpBuilder {
    ops: Vec<Bounds>,
}

impl ROpBuilder {
    /// Init a new builder.
    pub fn new() -> Self {
        ROpBuilder { ops: Vec::new() }
    }

    /// Add a new bound to the builder, this must be an expression of the form `expr op target`
    /// where expr is a math expression using R1,...,Rn and [supported expressions](https://docs.rs/meval/#supported-expressions),
    /// op is one of <, >, <=, >=, ==, != or ~ and target is an [f64 value](https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr).
    /// For ~ the bound will calculate the difference between the value of expr and target and add
    /// the abs error to the resulting error. For all other ops the bound will compare the value of
    /// expr to target, and if the comparison fails, it will reject the set of proposed values.
    pub fn bound(mut self, expr: &str) -> Self {
        self.ops.push(expr.parse().unwrap());
        self
    }

    fn cmp_bound_fn(&mut self) -> Box<Fn(&meval::Context) -> Option<f64>> {
        match self.ops.pop() {
            Some(b) => match b {
                Bounds::Cmp(op, expr, target) => {
                    let inner_bound = self.cmp_bound_fn();
                    Box::new(move |ctx| {
                        if op(expr.eval_with_context(ctx).unwrap(), target) {
                            inner_bound(ctx)
                        } else {
                            None
                        }
                    })
                }
                Bounds::Err(expr, target) => {
                    let inner_bound = self.cmp_bound_fn();
                    Box::new(move |ctx| {
                        let val = expr.eval_with_context(ctx).unwrap();
                        inner_bound(ctx).map(|v| v + (target - val).abs())
                    })
                }
            },
            None => Box::new(|_| Some(0.0)),
        }
    }

    /// Finishes the building and converts the struct into a function suitable to be passed to calc
    pub fn finish(mut self) -> impl Fn(&RSet) -> Option<f64> {
        let bound = self.cmp_bound_fn();
        move |rs: &RSet| {
            let mut ctx = meval::Context::new();
            for (i, v) in rs.0.iter().enumerate() {
                ctx.var(RNAMES[i].clone(), *v as f64);
            }
            bound(&ctx)
        }
    }
}
