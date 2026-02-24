use indexmap::IndexMap;
use crate::judger::*;

pub struct Chart<'a> { 
    pub highs: &'a [f32],
    pub lows: &'a [f32],
    merged: IndexMap<usize, i32>,
    peak_troughs: IndexMap<usize, Edge>,
    merged_bars: IndexMap<usize, (f32, f32)>,
    gaps: IndexMap<usize, (f32, f32)>,
    judger: Box<dyn Judger>,
}

impl<'a> Chart<'a> { 

    pub fn with_judger(highs: &'a [f32], lows: &'a [f32], judger: Box<dyn Judger>) -> Self {
        Chart { highs, lows, merged: IndexMap::new(), 
            merged_bars: IndexMap::new(), peak_troughs: IndexMap::new(), gaps: IndexMap::new(), judger }
    } 
    pub fn raw_with_judger(data_len: usize, ptr_highs: *mut f32, ptr_lows: *mut f32, judger: Box<dyn Judger>) -> Self {
        let highs = unsafe { std::slice::from_raw_parts(ptr_highs, data_len) };
        let lows = unsafe { std::slice::from_raw_parts(ptr_lows, data_len) };
        Chart::with_judger(highs, lows, judger)
    }

    pub fn merge(&mut self) { 
        let mut up = true;
        let mut prev_high = self.highs[0];
        let mut prev_low = self.lows[0];
        let mut merged_at = None;
        
        for i in 1..self.highs.len() { 
            let high = self.highs[i];
            let low = self.lows[i];

            if low > prev_high {
                self.gaps.insert(i, (low, prev_high));
            } else if high < prev_low {
                self.gaps.insert(i, (prev_low, high));
            }
            match (high.total_cmp(&prev_high), low.total_cmp(&prev_low), up) {
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater, true) |
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less, false) => {
                    // 延申无包含
                    prev_high = high;
                    prev_low = low;
                    merged_at = None;
                },
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater, false) | 
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less, true) => {
                    // 转向无包含
                    if self.peak_troughs.is_empty() && i > 1 { 
                        self.peak_troughs.insert(0, if up { Edge::Trough } else { Edge::Peak });
                    }
                    self.peak_troughs.insert(i - 1, if up { Edge::Peak } else { Edge::Trough });
                    if self.peak_troughs.len() > 1 {
                        let from = *self.peak_troughs.keys().nth(self.peak_troughs.len() - 2).unwrap();
                        let to = i - 1;
                        let high = if up { prev_high } else { self.get_high_at(from) };
                        let low = if up { self.get_low_at(from) } else { prev_low };
                        let count = self.bar_count(from, to);
                        self.judger.step(Stroke::new(from, to, count, high, low, up), &self.gaps);
                    }
                    prev_high = high;
                    prev_low = low;
                    merged_at = None;
                    up = !up;
                },
                (_, _, true) => {
                    // 向上包含
                    prev_high = high.max(prev_high);
                    prev_low = low.max(prev_low);
                    self.handle_merge(prev_high, prev_low, &mut merged_at, i);
                },
                (_, _, false) => {
                    // 向下包含
                    prev_high = high.min(prev_high);
                    prev_low = low.min(prev_low);
                    self.handle_merge(prev_high, prev_low, &mut merged_at, i);
                },
            }
        }
        if self.peak_troughs.len() > 1 {
            let from = *self.peak_troughs.keys().nth(self.peak_troughs.len() - 1).unwrap();
            let to = self.highs.len() - 1;
            let high = if up { prev_high } else { self.get_high_at(from) };
            let low = if up { self.get_low_at(from) } else { prev_low };
            let count = self.bar_count(from, to);
            self.judger.step(Stroke::new(from, to, count, high, low, up), &self.gaps);
        }
    }

    #[inline]
    fn handle_merge(&mut self, prev_high: f32, prev_low: f32, merged_at: &mut Option<usize>, i: usize) {
        let idx = i - 1;
        if !self.merged.contains_key(&idx) { 
            self.merged.insert(idx, 0);
            *merged_at = Some(idx);
        }
        self.merged_bars.insert(merged_at.unwrap(), (prev_high, prev_low));
        self.merged.insert(i, *self.merged.get(&idx).unwrap() + 1);
    }
    
    pub fn bar_count(&self, from: usize, to: usize) -> usize { 
        let mut count = 0;
        for i in from..=to { 
            if let Some(v) = self.merged.get(&i) {
                if i > from && *v > 0 { 
                    continue;
                }
            }
            count += 1;
        }
        count
    }

    pub fn get_low_at(&self, idx: usize) -> f32 { 
        match self.merged.get(&idx) {
            Some(n) if *n > 0 => {
                let merged_at = idx - *n as usize;
                self.merged_bars.get(&merged_at).unwrap().1
            },
            _ => self.lows[idx],
        }
    }

    pub fn get_high_at(&self, idx: usize) -> f32 { 
        match self.merged.get(&idx) {
            Some(n) if *n >= 0 => {
                let merged_at = idx - *n as usize;
                self.merged_bars.get(&merged_at).unwrap().0
            },
            _ => self.highs[idx],
        }
    }

    #[inline]
    pub fn get_vertexes(&mut self) -> Vec<Vertex> {
        self.judger.get_vertexes()
    }
    
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indexmap::IndexMap;
    use rstest::rstest;

    use crate::{chart::Chart, judger::{Edge, Judger, Stroke}};

    type ExpectedStrokes = Option<Vec<(usize, usize, usize, f32, f32, bool)>>;
    
    pub struct NullJudger {
        strokes: Vec<Stroke>,
    }

    impl NullJudger {
        pub(crate) fn with_capacity(len: usize) -> Self {
            Self { strokes: Vec::with_capacity(len) }
        }
    }

    impl Judger for NullJudger { 
        fn step(&mut self, stroke: Stroke, _gaps: &IndexMap<usize, (f32, f32)>) {  self.strokes.push(stroke); }
        fn get_strokes(&mut self) -> &Vec<Stroke> { &self.strokes }
    }

    impl<'a> Chart<'a> {
        pub fn new(highs: &'a [f32], lows: &'a [f32]) -> Self {
            Chart { highs, lows, merged: IndexMap::new(), 
                merged_bars: IndexMap::new(), peak_troughs: IndexMap::new(), gaps: IndexMap::new(), judger: Box::new(NullJudger::with_capacity(highs.len() / 3)) }
        }

        pub fn get_merged_bars(&self) -> Vec<(f32, f32)> { 
            let result = self.highs.iter().zip(self.lows.iter()).enumerate().filter_map(|m| {
                match self.merged.get(&m.0) {
                    Some(n) => { 
                        if *n > 0 { 
                            return None;
                        }
                        let merged_at = m.0 - *n as usize;
                        let (high, low) = self.merged_bars.get(&merged_at).unwrap();
                        Some((*high, *low))
                    },
                    _ => Some((*m.1.0, *m.1.1))
                }
            }).collect();
            result
        }

        pub fn get_peak_troughs(&self) -> HashMap<usize, Edge> { 
            self.peak_troughs.iter().map(|(idx, edge)| (*idx, *edge)).collect()
        }

        pub fn get_gaps(&self) -> IndexMap<usize, (f32, f32)> { 
            self.gaps.clone()
        }
        
        pub fn get_strokes(&mut self) -> &Vec<Stroke> { 
            self.judger.get_strokes()
        }

    }
    #[rstest]
    #[case::single_no_merge(vec![(10.0, 5.0)], None)]
    #[case::multiple_no_merge(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0), (22.0, 20.0)], None)]
    #[case::left_include(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 16.0)], Some(vec![(10.0, 5.0), (20.0, 16.0)]))]
    #[case::left_includes(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 16.0), (19.0, 17.0)], Some(vec![(10.0, 5.0), (20.0, 17.0)]))]
    #[case::right_include(vec![(10.0, 5.0), (20.0, 15.0), (28.0, 10.0)], Some(vec![(10.0, 5.0), (28.0, 15.0)]))]
    #[case::right_includes(vec![(10.0, 5.0), (20.0, 15.0), (28.0, 10.0), (30.0, 9.0)], Some(vec![(10.0, 5.0), (30.0, 15.0)]))]
    #[case::right_then_left_include(vec![(10.0, 5.0), (20.0, 15.0), (28.0, 10.0), (26.0, 16.0)], Some(vec![(10.0, 5.0), (28.0, 16.0)]))]
    #[case::up_and_down_includes(vec![(10.0, 5.0), (20.0, 15.0), (28.0, 10.0), (26.0, 10.0), (30.0, 9.0)], Some(vec![(10.0, 5.0), (28.0, 15.0), (26.0, 9.0)]))]
    fn test_merge_chart(#[case] bars: Vec<(f32, f32)>, #[case] expected: Option<Vec<(f32,f32)>>) {
        let (highs, lows): (Vec<_>, Vec<_>) = bars.clone().into_iter().unzip();
        let mut chart = super::Chart::new(&highs, &lows);
        chart.merge();

        let result = chart.get_merged_bars();
        if expected.is_none() {
            assert_eq!(result, bars);
        } else {
            assert_eq!(result, expected.unwrap());
        }
    }

    #[rstest]
    #[case::none(vec![(10.0, 5.0), (20.0, 15.0)], None)]
    #[case::single_peak(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0)], Some(vec![(0, -1), (1, 1)]))]
    #[case::up_n(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0), (22.0, 20.0)], Some(vec![(0, -1), (1, 1), (2, -1)]))]
    #[case::down_n(vec![(10.0, 5.0), (8.0, 3.0), (9.0, 4.0), (5.0, 2.0)], Some(vec![(0, 1), (1, -1), (2, 1)]))]
    fn test_peak_trough(#[case] bars: Vec<(f32, f32)>, #[case] expected: Option<Vec<(usize, i32)>>) {
        let (highs, lows): (Vec<_>, Vec<_>) = bars.clone().into_iter().unzip();
        let mut chart = super::Chart::new(&highs, &lows);
        chart.merge();

        let result = chart.get_peak_troughs();
        if expected.is_none() {
            assert!(result.is_empty());
        } else {
            let expected_edges: std::collections::HashMap<usize,super::Edge>  = expected.unwrap().into_iter().map(|(idx, edge)| {
                match edge {
                    1 => (idx, super::Edge::Peak),
                    -1 => (idx, super::Edge::Trough),
                    _ => panic!("Invalid edge value"),
                }
            }).collect();
            assert_eq!(result, expected_edges);
        }
    }

    #[rstest]
    #[case::single_gap(vec![(10.0, 5.0), (20.0, 15.0)], Some(vec![(1, (15.0, 10.0))]))]
    #[case::single_up_gap_with_peak(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0)], Some(vec![(1, (15.0, 10.0))]))]
    #[case::single_down_gap_with_peak(vec![(10.0, 5.0), (20.0, 10.0), (8.0, 5.0)], Some(vec![(2, (10.0, 8.0))]))]
    fn test_gaps(#[case] bars: Vec<(f32, f32)>, #[case] expected: Option<Vec<(usize, (f32, f32))>>) {
        let (highs, lows): (Vec<_>, Vec<_>) = bars.clone().into_iter().unzip();
        let mut chart = super::Chart::new(&highs, &lows);
        chart.merge();

        let result = chart.get_gaps();
        if expected.is_none() {
            assert!(result.is_empty());
        } else {
            assert_eq!(result, expected.unwrap().into_iter().collect::<indexmap::IndexMap<_,_>>());
        }
    }

    #[rstest]
    #[case::raw_peak_troughs(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0), (22.0, 20.0)], Some(vec![(0, 1, 2, 20.0, 5.0, true), (1, 2, 2, 20.0, 10.0, false), (2, 3, 2, 22.0, 10.0, true)]))]
    #[case::raw_peak_troughs_last_merged(vec![(10.0, 5.0), (20.0, 15.0), (18.0, 10.0), (22.0, 20.0), (21.0, 20.0)], Some(vec![(0, 1, 2, 20.0, 5.0, true), (1, 2, 2, 20.0, 10.0, false), (2, 4, 2, 22.0, 10.0, true)]))]
    fn test_raw_peak_troughs(#[case] bars: Vec<(f32, f32)>, #[case] expected: ExpectedStrokes) { 
        let (highs, lows): (Vec<_>, Vec<_>) = bars.clone().into_iter().unzip();
        let mut chart = super::Chart::new(&highs, &lows);
        chart.merge();

        let result = chart.get_strokes();
        if expected.is_none() {
            assert!(result.is_empty());
        } else {
            assert_eq!(result, &to_strokes(expected).unwrap());
        }
    }

    fn to_strokes(expected: ExpectedStrokes) -> Option<Vec<crate::chart::Stroke>> {
        Some(expected?.into_iter().map(|(start, end, count, high, low, up)| crate::chart::Stroke::new(start, end, count, high, low, up)).collect::<Vec<_>>())
    }

    #[rstest]
    #[case::default_strokes_up_and_down(vec![(2.0, 1.0), (2.5, 1.5), (3.0, 2.0), (3.5, 2.5), (4.0, 3.0), (4.5, 3.5), 
        (4.0, 3.0), (3.5, 2.5), (3.0, 2.0), (2.5, 1.5), (2.0, 1.0)], 
        Some(vec![(0, 5, 6, 4.5, 1.0, true), (5, 10, 6, 4.5, 1.0, false)]))]
    #[case::default_strokes_zup_and_zdown(vec![(2.0, 1.0), (2.5, 1.5), (2.2, 1.2), (3.5, 2.5), (3.0, 2.0), (4.5, 3.5), 
        (4.0, 3.0), (4.2, 3.1), (3.0, 2.0), (2.5, 1.5), (2.0, 1.0)], 
        Some(vec![(0, 5, 6, 4.5, 1.0, true), (5, 10, 6, 4.5, 1.0, false)]))]
    #[case::default_strokes_0down_then_zup_and_zdown(vec![(2.2, 1.2), (2.0, 1.0), (2.5, 1.5), (2.2, 1.2), (3.5, 2.5), (3.0, 2.0), (4.5, 3.5), 
        (4.0, 3.0), (4.2, 3.1), (3.0, 2.0), (2.5, 1.5), (2.0, 1.0)], 
        Some(vec![(1, 6, 6, 4.5, 1.0, true), (6, 11, 6, 4.5, 1.0, false)]))]
    #[case::cigao_todo(vec![(10.0, 9.0),(9.2, 8.2), (9.0, 8.0), (8.5, 7.5), (8.2, 7.2), (7.5, 6.5), (8.0, 7.0), (8.5, 7.5), 
        (9.0, 8.0), (8.9, 7.9), (8.8, 7.8), (8.7, 7.5), (8.0, 7.0), (7.8, 6.8), (7.9, 7.0), (8.0, 7.1), (8.2, 7.2), (8.3, 7.3), (8.4, 7.4), 
        (8.3, 7.3), (8.2, 7.2), (8.1, 7.1), (8.0, 7.0), (7.9, 6.0)], 
        Some(vec![(0, 5, 6, 10.0, 6.5, false), (5, 8, 4, 9.0, 6.5, true), (8, 13, 6, 9.0, 6.8, false), (13, 18, 6, 8.4, 6.8, true), (18, 23, 6, 8.4, 6.0, false)]))]
    fn test_default_strokes(#[case] bars: Vec<(f32, f32)>, #[case] expected: ExpectedStrokes) {
        use crate::judger::DefaultJudger;

        let (highs, lows): (Vec<_>, Vec<_>) = bars.clone().into_iter().unzip();
        let judger: Box<dyn super::Judger> = Box::new(DefaultJudger::new(false, 2));
        let mut chart = super::Chart::with_judger(&highs, &lows, judger);
        chart.merge();

        let result = chart.get_strokes();
        if expected.is_none() {
            assert!(result.is_empty());
        } else {
            assert_eq!(result, &to_strokes(expected).unwrap());
        }
    }

    #[test]
    fn test_strokes() {
        let highs = vec![
            25.44f32, 25.34, 25.19, 25.17, 25.25, 25.28, 25.36, 25.27, 25.30, 25.34, 25.25, 25.23,
            25.11, 25.09, 25.02, 25.12, 25.13, 25.20, 25.18, 25.16, 25.16, 25.20, 25.14, 25.16,
            25.14, 25.16, 25.19, 25.21, 24.96, 24.88, 24.92, 24.89, 24.88, 24.82, 24.80, 24.81,
            24.75, 24.79, 24.5];
        let lows = vec![
            25.21, 25.11, 25.05, 25.00, 25.08, 25.20, 25.19, 25.17, 25.21, 25.19, 25.17, 25.10,
            25.01, 24.98, 24.90, 24.97, 25.07, 25.07, 25.09, 25.10, 25.11, 25.14, 25.10, 25.05,
            25.00, 25.07, 25.13, 24.91, 24.76, 24.74, 24.79, 24.80, 24.69, 24.71, 24.72, 24.70,
            24.68, 24.48, 24.45];
        let judger: Box<dyn super::Judger> = Box::new(crate::judger::DefaultJudger::new(false, 2));
        let mut chart = super::Chart::with_judger(&highs, &lows, judger);

        chart.merge();
        let strokes = chart.get_strokes();
        assert_eq!(strokes.len(), 3);
    }

    #[test]
    fn test_samples() {
        let mut highs = [11.35, 11.07, 11.07, 11.05, 10.88, 10.92, 11.12, 11.09, 11.1, 10.7, 10.68, 10.81, 10.77, 11.01, 10.66, 10.7, 10.64, 10.48, 10.59, 10.71, 10.55, 10.47, 10.59, 10.6, 10.32, 10.77, 10.66, 10.86, 10.79, 10.75, 10.55, 10.46, 10.41, 10.36, 10.21, 10.25, 10.61, 10.7, 10.81, 10.79, 10.67, 10.51, 10.36, 10.4, 10.55, 10.76, 10.7, 10.76, 10.77, 10.78, 10.86, 10.95, 10.88, 10.79, 11.03, 11.03, 11.01, 10.87, 10.74, 10.62, 11.0, 11.1, 11.01, 11.14, 11.06, 10.96, 10.94, 10.79, 10.81, 10.76, 10.54, 10.22, 10.29, 10.24, 10.18, 10.17, 10.49, 11.01, 10.92, 10.72, 10.72, 10.63, 10.58, 10.59, 10.56, 10.56, 10.48, 10.5, 10.42, 10.48, 10.52, 10.54, 10.57, 10.71, 10.79, 10.84, 10.83, 10.83, 10.83, 10.67, 10.64, 10.62, 10.63, 10.49, 10.37, 10.77, 10.5, 10.64, 10.62, 10.71, 10.76, 10.66, 10.64, 10.52, 10.4, 10.22, 10.12, 10.17, 10.37, 10.32, 10.55, 10.68, 10.7, 10.67, 10.72, 10.66, 10.6, 10.45, 10.67, 10.72, 10.71, 10.64, 11.61, 12.17, 12.75, 12.53, 11.6, 11.06, 11.61, 11.48, 11.62, 11.58, 11.35, 11.31, 11.54, 11.64, 11.73, 11.87, 11.62, 11.44, 11.13, 10.99, 10.77, 10.9, 11.04, 10.94, 11.29, 11.9, 11.87, 11.82, 11.87, 11.97, 12.56, 12.26, 12.17, 12.22, 12.17, 12.67, 12.3, 12.45, 12.17, 12.11, 12.43, 12.5, 12.41, 12.56, 12.9, 13.15, 12.65, 12.68, 12.54, 12.79, 12.71, 12.57, 12.45, 12.39, 12.22, 12.04, 12.21, 12.39, 12.33, 12.19, 12.22, 11.96, 12.01, 11.81, 11.79, 11.58, 11.61, 11.55, 11.57, 11.79, 12.12, 12.29, 12.16, 12.15, 11.9, 11.81, 11.73, 11.69, 11.77, 11.67, 11.6, 11.56, 11.16, 11.1, 10.87, 10.14, 10.08, 10.41, 10.77, 10.6, 10.08, 9.65, 8.8, 8.87, 7.96, 6.96, 6.75, 6.67, 7.19, 7.3, 7.72, 7.76, 8.05, 8.38, 8.37, 8.75, 7.93, 8.05, 8.14, 7.99, 7.8, 7.87, 7.79, 7.87, 8.17, 8.21, 8.37, 8.3, 8.58, 8.9, 8.85, 8.93, 8.77, 8.59, 8.29, 8.3, 8.35, 8.59, 8.69, 8.82, 8.79, 8.7, 8.51, 8.52, 8.42, 8.56, 8.22, 7.57, 7.56, 8.15, 8.01, 7.91, 7.85, 7.92, 8.4, 8.58, 8.76, 8.83, 9.08, 9.2, 9.33, 9.42, 9.32, 9.1, 9.86, 9.52, 9.38, 9.28, 9.34, 9.28, 9.28, 9.14, 8.98, 9.06, 9.02, 8.86, 8.91, 8.85, 8.93, 8.62, 8.37, 8.21, 8.33, 8.33, 8.31, 8.33, 8.19, 8.07, 8.02, 8.05, 8.03, 7.93, 7.77, 7.58, 7.81, 7.92, 7.99, 7.96, 8.09, 8.05, 7.97, 7.95, 7.97, 7.76, 7.8, 8.01, 8.14, 8.21, 7.79, 7.98, 7.95, 7.96, 8.18, 8.19, 8.0, 8.0, 8.08, 8.13, 8.15, 8.17, 8.36, 8.53, 8.46, 8.66, 8.52, 8.57, 8.51, 8.59, 8.63, 8.72, 8.56, 8.49, 8.42, 8.27, 8.0, 7.98, 7.79, 7.79, 7.96, 8.03, 7.99, 8.23, 8.18, 8.1, 8.03, 8.12, 8.13, 8.11, 8.08, 7.97, 8.0, 7.87, 7.74, 7.81, 7.75, 7.7, 7.83, 8.08, 8.14, 8.53, 9.15, 9.98, 9.49, 9.1, 8.87, 8.88, 8.92, 8.94, 9.1, 9.06, 9.18, 9.35, 9.36, 9.38, 9.68, 10.39, 10.33, 9.79, 10.02, 9.72, 9.34, 9.62, 9.98, 10.33, 11.25, 11.43, 12.16, 11.99, 11.29, 11.35, 10.9, 10.6, 10.95, 11.26, 11.37, 11.52, 12.52, 11.9, 11.85, 11.73, 11.7, 12.07, 12.35, 12.18, 12.06, 12.59, 13.1, 13.52, 13.36, 12.96, 13.12, 13.15, 12.32, 12.07, 12.19, 12.13, 11.43, 11.17, 10.65, 10.66, 10.68, 10.85, 10.7, 10.43, 10.04, 10.05, 10.23, 9.95, 9.94, 9.74, 10.61, 10.72, 10.88, 10.87, 11.31, 11.36, 11.2, 11.17, 10.97, 11.14, 11.38, 11.27, 11.31, 11.23, 11.25, 11.32, 11.55, 11.21, 11.09, 11.03, 10.87, 11.13, 10.91, 10.62, 10.82, 10.6, 10.53, 10.4, 10.88, 10.81, 10.74, 10.52, 10.5, 10.47, 10.3, 10.71, 10.59, 11.06, 11.13, 11.2, 11.18, 11.11, 11.28, 10.89, 10.69, 11.0, 11.0, 11.08, 10.95, 11.34, 11.19, 11.24, 10.6, 9.95, 9.51, 10.12, 10.0, 10.37, 10.42, 10.27, 10.19, 10.25, 10.49, 10.57, 10.68, 10.67, 10.65, 10.45, 10.56, 10.64, 10.87, 11.76, 11.7, 11.45, 12.1, 12.17, 11.79, 11.5, 11.56, 11.62, 11.6, 11.69, 11.67, 11.97, 12.1, 13.3, 14.64, 16.11, 17.73, 19.51, 21.47, 23.63, 22.99, 23.34, 24.29, 24.9, 25.5, 24.7, 24.79, 25.31, 23.4, 21.91, 20.74, 19.49, 19.46, 19.2, 19.4, 19.3, 19.21, 18.94, 18.56, 18.44, 18.18, 18.3, 18.11, 18.05, 18.35, 18.41, 19.39, 18.69, 19.92, 20.9, 22.99, 23.73, 22.66, 22.62, 22.73, 22.54, 22.19, 22.32, 22.16, 21.88, 21.29, 22.81, 23.9, 23.14, 24.54, 24.3, 24.36, 25.4, 26.83, 25.34, 22.55, 22.35, 22.3, 21.43, 20.59, 20.58, 20.38, 20.1, 19.69, 19.85, 19.96, 20.28, 19.68, 20.35, 20.03, 20.03, 19.67, 19.5, 19.63, 21.2, 21.04, 20.57, 20.05, 20.54, 20.4, 20.09, 19.69, 19.65, 19.3, 19.4, 18.93, 19.38, 19.03, 19.05, 19.1, 18.87, 19.18, 19.44, 19.36, 20.35, 20.3, 20.33, 21.0, 21.0, 20.29, 20.42, 20.45, 20.4, 20.2, 20.09, 20.57, 20.33, 20.42, 20.55, 20.54, 20.2, 19.95, 19.76, 19.4, 19.33, 18.9, 19.23, 19.19, 18.96, 18.99, 19.12, 18.66, 18.48, 18.38, 18.37, 18.58, 18.73, 18.2, 18.19, 17.98, 18.34, 18.1, 17.91, 18.19, 18.38, 18.68, 18.46, 18.18, 18.14, 18.15, 18.12, 18.2, 18.77, 18.51, 18.74, 19.37, 18.66, 18.43, 18.4, 19.5, 19.18, 18.73, 18.48, 19.5, 20.0, 19.65, 19.55, 19.6, 19.55, 19.45, 19.1];
        let mut lows = [10.96, 10.92, 10.97, 10.82, 10.6, 10.51, 10.69, 10.79, 10.64, 10.35, 10.36, 10.68, 10.56, 10.58, 10.47, 10.51, 10.4, 10.24, 10.29, 10.47, 10.37, 10.29, 10.32, 10.12, 10.16, 10.18, 10.21, 10.57, 10.55, 10.49, 10.36, 10.05, 10.19, 10.02, 9.64, 9.77, 10.1, 10.39, 10.62, 10.47, 10.39, 10.18, 10.08, 10.16, 10.19, 10.28, 10.47, 10.47, 10.57, 10.49, 10.69, 10.66, 10.7, 10.52, 10.56, 10.86, 10.62, 10.6, 10.46, 10.43, 10.47, 10.84, 10.77, 10.87, 10.81, 10.62, 10.6, 10.61, 10.54, 10.44, 10.11, 10.07, 10.05, 9.82, 9.82, 9.93, 10.12, 10.33, 10.6, 10.58, 10.56, 10.46, 10.43, 10.37, 10.39, 10.38, 10.32, 10.36, 10.18, 10.31, 10.39, 10.43, 10.37, 10.43, 10.61, 10.74, 10.66, 10.64, 10.59, 10.55, 10.52, 10.5, 10.45, 10.11, 10.08, 10.28, 10.29, 10.3, 10.32, 10.42, 10.51, 10.46, 10.38, 10.23, 9.87, 9.99, 9.91, 9.82, 10.03, 10.0, 10.33, 10.38, 10.49, 10.51, 10.58, 10.52, 10.34, 10.33, 10.31, 10.57, 10.4, 10.48, 10.48, 11.14, 11.28, 11.46, 10.93, 10.65, 10.68, 11.14, 11.27, 11.29, 11.06, 10.97, 10.98, 11.3, 11.35, 11.48, 11.31, 11.07, 10.79, 10.68, 10.43, 10.49, 10.8, 10.77, 10.58, 11.25, 11.58, 11.58, 11.62, 11.75, 11.79, 11.78, 11.74, 11.89, 11.77, 11.9, 11.98, 12.01, 11.93, 11.88, 12.09, 12.2, 12.2, 12.2, 12.39, 12.41, 12.17, 12.35, 12.21, 12.4, 12.4, 12.28, 12.18, 12.09, 11.82, 11.68, 11.95, 11.99, 12.02, 11.93, 11.87, 11.7, 11.74, 11.47, 11.49, 11.34, 11.33, 11.32, 11.26, 11.47, 11.71, 11.97, 11.97, 11.69, 11.54, 11.55, 11.48, 11.42, 11.41, 11.35, 11.22, 11.15, 10.65, 10.78, 9.97, 9.69, 9.5, 9.91, 10.43, 10.07, 9.53, 8.67, 8.38, 7.76, 7.11, 6.39, 6.04, 5.44, 6.66, 6.88, 7.07, 7.35, 7.67, 7.94, 8.13, 7.52, 7.32, 7.69, 7.83, 7.65, 7.58, 7.67, 7.63, 7.65, 7.89, 8.0, 7.94, 8.03, 8.21, 8.53, 8.53, 8.63, 8.41, 8.21, 8.01, 8.04, 7.98, 8.24, 8.42, 8.61, 8.59, 8.23, 8.16, 8.15, 8.1, 8.2, 7.48, 6.85, 6.91, 7.65, 7.77, 7.41, 7.48, 7.7, 7.87, 8.37, 8.46, 8.65, 8.81, 8.98, 9.05, 9.06, 8.94, 8.9, 8.95, 9.23, 9.13, 9.07, 9.13, 9.12, 9.08, 8.75, 8.72, 8.78, 8.68, 8.59, 8.66, 8.66, 8.56, 8.3, 8.11, 7.62, 7.85, 7.96, 8.01, 8.05, 8.01, 7.83, 7.78, 7.89, 7.67, 7.55, 7.36, 7.35, 7.42, 7.67, 7.65, 7.67, 7.83, 7.9, 7.52, 7.45, 7.6, 7.36, 7.57, 7.72, 7.94, 7.66, 7.61, 7.63, 7.53, 7.82, 7.93, 7.98, 7.83, 7.73, 7.91, 7.97, 7.95, 7.85, 8.14, 8.2, 8.1, 8.17, 8.36, 8.37, 8.32, 8.28, 8.38, 8.46, 8.41, 8.37, 8.14, 7.83, 7.82, 7.75, 7.53, 7.54, 7.78, 7.7, 7.82, 7.95, 7.92, 7.87, 7.83, 7.86, 7.93, 7.84, 7.84, 7.82, 7.85, 7.71, 7.43, 7.5, 7.52, 7.52, 7.6, 7.88, 7.95, 8.14, 8.47, 8.98, 8.6, 8.62, 8.46, 8.46, 8.7, 8.64, 8.8, 8.68, 8.75, 8.98, 9.11, 9.16, 9.21, 9.44, 9.66, 9.35, 9.5, 9.09, 9.06, 9.26, 9.41, 9.52, 10.25, 10.57, 11.18, 11.07, 10.9, 10.7, 10.2, 10.21, 10.45, 10.8, 10.85, 10.84, 11.47, 11.25, 11.56, 11.45, 11.46, 11.54, 11.83, 11.83, 10.81, 11.7, 12.27, 12.35, 12.8, 12.61, 12.61, 11.97, 11.71, 11.64, 11.75, 11.26, 10.99, 10.11, 10.24, 10.35, 10.1, 10.26, 10.25, 9.75, 9.45, 9.7, 9.55, 9.68, 9.61, 9.27, 9.69, 10.28, 10.33, 10.56, 10.64, 10.98, 10.81, 10.87, 10.74, 10.88, 10.98, 11.0, 10.89, 11.01, 11.05, 11.11, 11.16, 10.86, 10.83, 10.68, 10.63, 10.72, 10.5, 10.23, 10.4, 10.3, 10.03, 10.04, 10.13, 10.44, 10.17, 10.3, 10.15, 10.19, 10.11, 10.25, 10.1, 10.21, 10.72, 10.96, 11.0, 10.92, 10.77, 10.15, 10.3, 10.56, 10.65, 10.78, 10.48, 10.67, 10.9, 10.92, 9.93, 8.93, 8.08, 9.44, 9.71, 9.89, 10.16, 9.77, 9.96, 9.92, 10.02, 10.38, 10.47, 10.41, 10.3, 10.16, 10.3, 10.46, 10.6, 10.74, 11.06, 11.13, 11.23, 11.5, 11.37, 11.28, 11.28, 11.31, 11.45, 11.45, 11.4, 11.58, 11.63, 13.3, 14.64, 16.11, 17.73, 19.51, 21.47, 21.48, 21.78, 22.02, 22.42, 22.6, 23.2, 22.67, 23.16, 22.73, 20.72, 20.1, 18.97, 18.79, 18.62, 18.71, 18.81, 18.75, 18.4, 18.49, 18.21, 17.93, 17.79, 17.81, 17.53, 17.72, 17.99, 17.88, 18.1, 17.97, 18.01, 18.65, 21.1, 21.79, 21.7, 21.35, 21.5, 21.26, 21.44, 21.04, 21.2, 20.91, 20.65, 20.67, 22.11, 22.23, 22.22, 22.67, 23.21, 24.16, 23.9, 23.17, 21.38, 21.72, 21.11, 20.51, 19.38, 19.87, 19.8, 19.12, 18.88, 18.93, 18.98, 19.56, 19.11, 19.28, 19.4, 19.59, 19.25, 19.09, 19.18, 19.27, 20.21, 19.82, 19.42, 19.74, 19.41, 18.76, 19.3, 19.05, 18.63, 18.83, 18.31, 18.32, 18.37, 18.56, 18.35, 18.37, 18.7, 19.0, 18.75, 19.1, 19.6, 19.64, 20.12, 20.07, 19.79, 20.0, 20.08, 19.96, 19.76, 19.76, 19.84, 19.74, 20.08, 19.95, 20.0, 19.7, 19.59, 19.0, 19.01, 18.3, 18.32, 18.68, 18.75, 18.62, 18.6, 18.49, 18.2, 18.15, 18.01, 17.97, 18.18, 18.01, 17.95, 17.82, 17.58, 17.45, 17.52, 17.42, 17.64, 17.96, 18.11, 17.9, 17.92, 17.99, 17.96, 17.81, 17.95, 17.99, 18.13, 18.4, 18.3, 18.36, 18.08, 18.06, 18.23, 18.41, 18.35, 18.16, 18.28, 19.0, 19.03, 19.18, 18.85, 18.8, 18.63, 18.49];

        let judger: Box<dyn super::Judger> = Box::new(crate::judger::DefaultJudger::new(false, 2));
        let count = 152;
        let start = highs.len() - count;
        let mut chart = super::Chart::raw_with_judger(count, highs[start..].as_mut_ptr(), lows[start..].as_mut_ptr(), judger);

        chart.merge();
        let strokes = chart.get_strokes();
        println!("{:?}", strokes);
        assert!(!strokes.is_empty());
        let vertex = chart.get_vertexes();
        println!("{:?}", vertex);
    }

}