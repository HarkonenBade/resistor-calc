extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate meval;

use itertools::Itertools;

use std::fmt;
use std::str::FromStr;

const POWERS: &[f64] = &[1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6];

lazy_static! {
    pub static ref E3: RSeries = RSeries::new(&[1.0, 2.2, 4.7]);
    pub static ref E6: RSeries = RSeries::new(&[1.0, 1.5, 2.2, 3.3, 4.7, 6.8]);
    pub static ref E12: RSeries =
        RSeries::new(&[1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2]);
    pub static ref E24: RSeries = RSeries::new(&[
        1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0, 3.3, 3.6, 3.9, 4.3, 4.7, 5.1,
        5.6, 6.2, 6.8, 7.5, 8.2, 9.1,
    ]);
    static ref RNAMES: Vec<String> = (1..=100).map(|i| format!("R{}", i)).collect();
}

#[derive(Debug)]
pub struct RSeries {
    values: Box<[f64]>,
}

impl RSeries {
    fn new(series: &[f64]) -> Self {
        RSeries {
            values: series
                .iter()
                .cartesian_product(POWERS.iter())
                .map(|(val, pow)| val * pow)
                .collect::<Vec<f64>>()
                .into_boxed_slice(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &f64> + Clone {
        self.values.iter()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

fn _format_rval(r: f64, unit: &str) -> String {
    let mut val = format!("{}", r);
    if val.contains('.') {
        val.replace(".", unit)
    } else {
        val.push_str(unit);
        val
    }
}

fn _print_r(r: &f64) -> String {
    if *r < 1000.0 {
        _format_rval(*r, "R")
    } else if *r < 1_000_000.0 {
        _format_rval(*r / 1000.0, "K")
    } else {
        _format_rval(*r / 1_000_000.0, "M")
    }
}

fn _print_res(r: &(u64, RSet)) {
    let &(r, ref v) = r;
    println!("Error: {:.3}\nValues: {}", (r as f64) / 1e9, v);
}

#[derive(Debug)]
pub struct RSet(Box<[f64]>);

impl RSet {
    pub fn r(&self, idx: usize) -> f64 {
        self.0[idx - 1]
    }

    pub fn sum(&self) -> f64 {
        self.0.iter().sum()
    }
}

impl fmt::Display for RSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sep = if f.alternate() { "\n" } else { ", " };
        write!(
            f,
            "{}",
            self.0
                .iter()
                .enumerate()
                .map(|(i, r)| format!("R{}: {}", i + 1, _print_r(r)))
                .join(sep)
        )
    }
}

#[derive(Debug)]
pub struct RRes {
    res: Vec<(u64, RSet)>,
}

impl RRes {
    pub fn print_best(&self) {
        let best_err = self.res[0].0;
        for (idx, res) in self.res
            .iter()
            .take_while(|(err, _)| *err == best_err)
            .enumerate()
        {
            println!("Match {}:", idx + 1);
            _print_res(res);
            println!();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(u64, RSet)> {
        self.res.iter()
    }
}

#[derive(Debug)]
pub struct RCalc {
    rs: Vec<&'static RSeries>,
}

impl RCalc {
    pub fn new(rs: Vec<&'static RSeries>) -> Self {
        RCalc { rs }
    }

    pub fn combinations(&self) -> u128 {
        self.rs.iter().map(|r| r.len() as u128).product()
    }

    pub fn calc(&self, f: impl Fn(&RSet) -> Option<f64>) -> Option<RRes> {
        let mut res: Vec<(u64, RSet)> = self.rs
            .iter()
            .map(|r| r.iter().cloned())
            .multi_cartesian_product()
            .filter_map(|v| {
                let rs = RSet(v.into_boxed_slice());
                f(&rs).map(|err| ((err * 1e9).round() as u64, rs))
            })
            .collect();
        res.sort_by_key(|(err, _rs)| *err);
        if !res.is_empty() {
            Some(RRes { res })
        } else {
            None
        }
    }
}

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
                Box::new(|a, b| (a - b).abs() < std::f64::EPSILON),
                ex,
                trg,
            ))
        } else if s.contains("!=") {
            let (ex, trg) = split_expr(s, "!=");
            Ok(Bounds::Cmp(
                Box::new(|a, b| (a - b).abs() > std::f64::EPSILON),
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

#[derive(Default)]
pub struct ROpBuilder {
    ops: Vec<Bounds>,
}

impl ROpBuilder {
    pub fn new() -> Self {
        ROpBuilder { ops: Vec::new() }
    }

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

fn main() {
    let rcalc = RCalc::new(vec![&E24, &E6, &E24]);

    println!("Number of combinations: {}", rcalc.combinations());

    let res = rcalc
        .calc(
            ROpBuilder::new()
                .bound("R1+R2+R3 <= 1e6")
                .bound("R1+R2+R3 >= 1e4")
                .bound("0.8 * (1 + R1/R3) ~ 6.0")
                .bound("0.8 * (1 + (R1+R2)/R3) ~ 12.0")
                .finish(),
        )
        .expect("Error: No values satisfy requirements");

    res.print_best();
}
