[![Crates.io](https://img.shields.io/crates/v/resistor-calc.svg)](https://crates.io/crates/resistor-calc/)
[![Docs.rs](https://docs.rs/resistor-calc/badge.svg)](https://docs.rs/resistor-calc/)

A resistor value optimiser for circuit design.

When provided with a set of constraints and relations for a series of resistors R1, R2, ..., it
can present sets of values from standard series in order of increasing inaccuracy.

# Example
Given the following resistor network:

![diagram](https://i.imgur.com/GoZKJoL.png)

Where VADJ must remain at 0.8v, as R2 varies from no to full resistance,
VOUT varies from 6v to 12v

We can then describe the problem via the following constraints, plus a few extra bounds to
eliminate either very small, or very large values, both of which may cause current issues.
```rust
extern crate resistor_calc;

use resistor_calc::*;

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
```
Running this example produces the results:
```text
Number of combinations: 1185408
Match 1:
Error: 0.000
Values: R1: 13K, R2: 15K, R3: 2K

Match 2:
Error: 0.000
Values: R1: 130K, R2: 150K, R3: 20K
```