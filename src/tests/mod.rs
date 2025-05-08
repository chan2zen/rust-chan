use crate::market::{Pole, STEPS, Edge};

macro_rules! create_market {
    ($cnt:expr, [$($high_vals:expr),*], [$($low_vals:expr),*]) => {
        {
            let mut high = vec![$($high_vals),*];
            let mut low = vec![$($low_vals),*];
            let len = if $cnt > 0 { $cnt } else { high.len() };
            Market::new(len, high.as_mut_ptr(), low.as_mut_ptr())
        }
    };
}

macro_rules! create_market_with_bimode {
    ($cnt:expr, $bimode:expr, [$($high_vals:expr),*], [$($low_vals:expr),*]) => {
        {
            let mut high = vec![$($high_vals),*];
            let mut low = vec![$($low_vals),*];
            let len = if $cnt > 0 { $cnt } else { high.len() };
            Market::with_bi_mode(len, high.as_mut_ptr(), low.as_mut_ptr(), $bimode)
        }
    };
}

fn to_poles(pole_values: &[f32]) -> Vec<Pole> {
    let mut poles = Vec::with_capacity(pole_values.len());
    let mut edge = Edge::TROUGH;
    for i in 1..=pole_values.len() {
        let i = i-1;
        if i == 0 && pole_values[i] > pole_values[i + 1] {
            edge = Edge::PEAK;
        }
        poles.push(Pole::new(i * STEPS, edge, pole_values[i], false));
        edge = edge.oppsite();
    }
    poles
}

mod market_tests;
mod forest_tests;
mod pivot_tests;