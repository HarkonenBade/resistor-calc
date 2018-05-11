extern crate itertools;
#[macro_use]
extern crate lazy_static;

use itertools::Itertools;

use std::fmt;

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
}

#[derive(Debug)]
pub struct RSeries {
    values: Box<[u64]>,
}

impl RSeries {
    fn new(series: &[f64]) -> Self {
        RSeries {
            values: series
                .iter()
                .cartesian_product(POWERS.iter())
                .map(|(val, pow)| (val * pow) as u64)
                .collect::<Vec<u64>>()
                .into_boxed_slice(),
        }
    }

    fn iter(&self) -> impl Iterator<Item = &u64> + Clone {
        self.values.iter()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

fn _print_r(r: &u64) -> String {
    if *r < 1000 {
        format!("{}R", r)
    } else if *r < 1000000 {
        format!("{}K", r / 1000)
    } else {
        format!("{}M", r / 1000000)
    }
}

fn _print_res(r: &(u64, RSet)) {
    let &(r, ref v) = r;
    println!("Error: {:.3}\nValues: {}", (r as f64) / 1e9, v);
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct RSet(Box<[u64]>);

impl RSet {
    pub fn r(&self, idx: usize) -> f64 {
        self.0[idx - 1] as f64
    }

    pub fn sum(&self) -> u64 {
        self.0.iter().sum()
    }
}

impl fmt::Display for RSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sep = if f.alternate() {
            "\n"
        } else {
            ", "
        };
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
        self.rs
            .iter()
            .map(|r| r.len() as u128)
            .fold(1_u128, |acc, i| acc * i)
    }

    pub fn calc(&self, f: impl Fn(&RSet) -> Option<f64>) -> Option<RRes> {
        let mut res: Vec<(u64, RSet)> = self.rs
            .iter()
            .map(|r| r.iter().map(|v| *v))
            .multi_cartesian_product()
            .filter_map(|v| {
                let rs = RSet(v.into_boxed_slice());
                f(&rs).map(|err| ((err * 1e9).round() as u64, rs))
            })
            .collect();
        res.sort();
        if res.len() > 0 {
            Some(RRes { res })
        } else {
            None
        }
    }
}

fn main() {
    let r = RCalc::new(vec![&E24, &E6, &E24]);

    println!("Number of combinations: {}", r.combinations());

    let res = r.calc(|rs| {
        if rs.sum() < 10000 || rs.sum() > 1000000 {
            None
        } else {
            let six_err = (6.0 - (0.8 * (1.0 + rs.r(1) / rs.r(3)))).abs();
            let twelve_err = (12.0 - (0.8 * (1.0 + (rs.r(1) + rs.r(2)) / rs.r(3)))).abs();
            Some(six_err + twelve_err)
        }
    }).expect("Error: No values satisfy requirements");

    res.print_best();
}
