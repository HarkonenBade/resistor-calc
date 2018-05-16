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