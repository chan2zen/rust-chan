use crate::{market::*, tests::to_poles};

fn assert_forest_eq(expected: State, segmented: &[usize], pole_values: &[f32]) {
    let mut forest = Forest::new();
    let poles = to_poles(pole_values);
    for pole in poles {
        forest.step(&pole);
    }
    let indexes = forest.indexes();
    let segmented = if segmented.len() < 3 { &[] } else { &segmented[0..&segmented.len()-1] };
    assert_eq!(segmented.iter().map(|s| *s * STEPS).collect::<std::collections::HashSet<usize>>(), indexes);
    assert_eq!(expected, forest.state());
}

#[test]
fn go_none() {
    assert_forest_eq(State::None, &[],&[1.0, 3.0, 2.0]);
}

#[test]
fn go_none_2() {
    assert_forest_eq(State::None, &[],&[1.0, 3.0, 2.0, 2.5]);
}

#[test]
fn go_0() {
    assert_forest_eq(State::S0, &[0, 3],&[1.0, 3.0, 2.0, 5.0]);
}

#[test]
fn go_0_1() {
    assert_forest_eq(State::S0, &[1, 4],&[6., 1.0, 3.0, 2.0, 5.]);
}


#[test]
fn go_0_2() {
    assert_forest_eq(State::S0, &[2, 5],&[1.0, 3.0, 2.0, 2.8, 2.5, 5.]);
}

#[test]
fn go_1() {
    assert_forest_eq(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0]);
}

#[test]
fn go_2() {
    assert_forest_eq(State::S2, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5]);
}

#[test]
fn go_11() {
    assert_forest_eq(State::S0, &[0, 5],&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0]);
}

#[test]
fn go_12() {
    assert_forest_eq(State::S12, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5]);
}

#[test]
fn go_21() {
    assert_forest_eq(State::S0, &[0, 5],&[1.0, 3.0, 2.0, 5.0, 2.5, 6.0]);
}
#[test]
fn go_22() {
    assert_forest_eq(State::S22, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5]);
}

#[test]
fn go_121() {
    assert_forest_eq(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 4.2]);
}

#[test]
fn go_122() {
    assert_forest_eq(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5]);
}

#[test]
fn go_123() {
    assert_forest_eq(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5]);
}
#[test]
fn go_124() {
    assert_forest_eq(State::S0, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 0.5]);
}


#[test]
fn go_221() {
    assert_forest_eq(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 4.2]);
}

#[test]
fn go_222() {
    assert_forest_eq(State::S2, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 2.8]);
}

#[test]
fn go_223() {
    assert_forest_eq(State::S0, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 2.2]);
}

#[test]
fn go_1221() {
    assert_forest_eq(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8]);
}

#[test]
fn go_1222() {
    assert_forest_eq(State::S1222, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2]);
}

#[test]
fn go_1223() {
    assert_forest_eq(State::S12, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 4.6]);
}
#[test]
fn go_1224() {
    assert_forest_eq(State::S0, &[0, 7],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 6.]);
}

#[test]
fn go_1231() {
    assert_forest_eq(State::S1231,&[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8]);
}

#[test]
fn go_1232() {
    assert_forest_eq(State::S1232, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2]);
}

#[test]
fn go_1233() {
    assert_forest_eq(State::S22, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.8]);
}
#[test]
fn go_1234() {
    assert_forest_eq(State::S0, &[0, 7],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 6.0]);
}

#[test]
fn go_12211() {
    assert_forest_eq(State::S12211, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6]);
}

#[test]
fn go_12212() {
    assert_forest_eq(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.4]);
}
#[test]
fn go_12213() {
    assert_forest_eq(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 2.9]);
}
#[test]
fn go_12214() {
    assert_forest_eq(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 0.9]);
}
#[test]
fn go_12221() {
    assert_forest_eq(State::S12221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9]);
}
#[test]
fn go_12222() {
    assert_forest_eq(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.2]);
}
#[test]
fn go_12223() {
    assert_forest_eq(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 2.9]);
}
#[test]
fn go_12224() {
    assert_forest_eq(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 0.9]);
}
#[test]
fn go_12311() {
    assert_forest_eq(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 1.8]);
}
#[test]
fn go_12312() {
    assert_forest_eq(State::S12312, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2]);
}

#[test]
fn go_12321() {
    assert_forest_eq(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 2.0]);
}
#[test]
fn go_12322() {
    assert_forest_eq(State::S12322, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5]);
}

#[test]
fn go_122111() {
    assert_forest_eq(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 3.7]);
}
#[test]
fn go_122112() {
    assert_forest_eq(State::S122, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 3.9]);
}
#[test]
fn go_122113() {
    assert_forest_eq(State::S123, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 4.3]);
}
#[test]
fn go_122114() {
    assert_forest_eq(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 6.]);
}

#[test]
fn go_122211() {
    assert_forest_eq(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 3.95]);
}
#[test]
fn go_122212() {
    assert_forest_eq(State::S1222, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 4.1]);
}
#[test]
fn go_122213() {
    assert_forest_eq(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 4.6]);
}

#[test]
fn go_123121() {
    assert_forest_eq(State::S1231, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 3.6]);
}

#[test]
fn go_123122() {
    assert_forest_eq(State::S122, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 3.9]);
}

#[test]
fn go_123123() {
    assert_forest_eq(State::S123, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 4.8]);
}

#[test]
fn go_123124() {
    assert_forest_eq(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 6.0]);
}

#[test]
fn go_123221() {
    assert_forest_eq(State::S1231, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 3.8]);
}

#[test]
fn go_123222() {
    assert_forest_eq(State::S1232, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 4.1]);
}

#[test]
fn go_123223() {
    assert_forest_eq(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 4.6]);
}

#[test]
fn go_origin_sample() {
    assert_forest_eq(State::S0, &[0, 5, 14, 19],&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0, 4.5, 5.5, 3.2, 4.8, 1.5, 6.2, 1.2, 3.5, 0.5, 1.3, 0.8, 5.6, 3.0, 7.]);
}

#[test]
fn go_reverse_nostandard_fenxing() {
    todo!()
}