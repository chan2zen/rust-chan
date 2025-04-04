use crate::{market::{Forest, PivotFinder, State, STEPS}, tests::to_poles};

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
    let pivots = forest.pivots(&poles);
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

#[test]
fn tjbfj_empty() {
    let finder = PivotFinder::new();
    let samples = [1., 2., 1.5, 2.5, 2.2, 3.0, 2.7, 3.5, 3.2];
    let len = samples.len();
    for i in 0..len {
        let poles = to_poles(&samples[0..i]);
        let pivots = finder.find(&poles);
        assert!(pivots.is_empty());
    }
}

#[test]
fn tjbfj_simple() {
    let finder = PivotFinder::new();
    for last in [2.0, 1.7, 1.5, 1.0, 0.5] {
        let mut samples = [1.0f32, 2., 1.5, 2.5].to_vec();
        samples.push(last);
        let poles = to_poles(samples.as_slice());
        let pivots = finder.find(&poles);
        assert!(!pivots.is_empty(), "last: {last}");
    }
}