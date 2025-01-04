use crate::{market::{Forest, State, STEPS}, tests::to_poles};

fn assert_forest_eq(expected: State, segmented: &[usize], pole_values: &[f32]) {
    let mut forest = Forest::new();
    let mut poles = to_poles(pole_values);
    for pole in &poles {
        forest.step(&pole);
    }
    let indexes = forest.indexes();
    for pole in poles.as_mut_slice() {
        if indexes.contains(&pole.index) {
            (*pole).segmented = true;
        }
    }
    let (_signals, pivots) = forest.pivots(&poles);
    println!("pivots: {:?}", pivots);
    for p in pivots {
        println!("pivot: {}-{}, high:{}, low:{}", p.start(), p.end(), p.high(), p.low());
    }
    let segmented = if segmented.len() < 3 { &[] } else { &segmented[0..&segmented.len()-1] };
    assert_eq!(segmented.iter().map(|s| *s * STEPS).collect::<std::collections::HashSet<usize>>(), indexes);
    assert_eq!(expected, forest.state());
}
#[test]
fn test_pivot() {
    assert_forest_eq(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 4.6]);
}

