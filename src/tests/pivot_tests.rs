use crate::{market::*, tests::to_poles};

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
fn test_pivot_empty() {
    let samples = [10., 11., 9., 9.5, 8., 8.5, 8.2, 9.8, 8.8, 10.];
    do_pivot_test(&samples);
}
#[test]
fn test_pivot_empty_multi_a0() {
    let samples = [11., 9., 9.5, 8., 10.9, 10., 12., 11.1, 12., 10., 11., 9., 9.5, 8., 8.9, 8.5, 9.5, 9., 10.];
    do_pivot_test(&samples);
}
#[test]
fn test_pivot_1_pivot() {
    let samples = [10., 11., 9., 9.5, 8., 8.5, 8.2, 8.8, 8.3, 8.7];
    do_pivot_test(&samples);
}
#[test]
fn test_pivot_1_pivot_3s() {
    let samples = [10., 11., 9., 9.5, 8., 8.5, 8.2, 8.8, 8.3, 8.7, 6., 7., 5.];
    do_pivot_test(&samples);
}

#[test]
fn test_pivot_2_down() {
    let samples = [15., 10., 11., 9., 10.5, 8., 8.5, 7., 8.8, 5., 5.5, 4.];
    do_pivot_test(&samples);
}


#[test]
fn test_pivot_1_extended() {
    let samples = [15., 10., 12., 9., 11., 8., 11.5, 10.2, 11.3, 9.5, 10.8];
    do_pivot_test(&samples);
}

#[test]
fn test_pivot_1_extended_3b() {
    let samples = [15., 10., 12., 9., 11., 8., 11.5, 10.2, 11.3, 9.5, 14., 13.];
    do_pivot_test(&samples);
}
     
fn do_pivot_test(samples: &[f32]) {
    let poles = to_poles(&samples);
    do_pivot_test_with_poles(poles);
}

fn do_pivot_test_with_poles(poles: Vec<Pole>) {
    let finder = PivotFinder::new();
    
    let entries = finder.find(&poles);
    for entry in &entries {
        let path = entry.entry.iter().map(|p| p.value).collect::<Vec<_>>();
        print!("\nentry: {:?}, signals: {:?}, - pivot:", path, entry.signals);
        if let Some(pivot) = &entry.pivot {
            println!("extended: {}, {:?}", pivot.extended, pivot.poles.iter().map(|p| p.value).collect::<Vec<_>>());
        }
    }
}
#[test]
fn sample() {
    let market = include!("demo.rs");
    
    let mut poles = market.tracer.poles();
    println!("poles: {:?}", poles.iter().map(|p| p.value).collect::<Vec<f32>>());
    market.stain_duan(&mut poles);
    println!("stained poles: {:?}", poles.iter().map(|p| p.value).collect::<Vec<f32>>());
    do_pivot_test_with_poles(poles);
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