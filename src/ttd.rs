
#[cfg(test)]
mod tests {
    use crate::chan::*;
    use crate::chan::Trend::{Advance, Decline};

    #[test]
    fn split_one() {
        // 图一: 无缺口情况 406 页
        test_split(vec![
            Stroke{low:1.0, high:3.0, from: 0, to:5, trend:Trend::Advance},
            Stroke{high:3.0, low:2.0, from: 5, to:10, trend:Trend::Decline},
            Stroke{low:2.0, high: 5.0, from:10, to: 15, trend:Trend::Advance},
            Stroke{high:5.0, low:2.5, from: 15, to:20, trend:Trend::Decline},
            Stroke{low:2.5, high:4.5, from: 20, to:25, trend:Trend::Advance},
            Stroke{high:4.5, low:1.5, from: 25, to: 30, trend:Trend::Decline}], 
            vec![
                Pole{index: 0, frac: -1, count: 1, value: 1.0},
                Pole{index: 15, frac: 1, count:1, value: 5.0},
                Pole{index:30, frac: -1, count:1, value: 1.5}
            ]);
    }

    #[test]
    fn split_one_qk() {
        // 图一: 有缺口情况但是线段封闭 406 页
        test_split(vec![
            Stroke{low:1.0, high:3.0, from: 0, to:5, trend:Trend::Advance},
            Stroke{high:3.0, low:2.0, from: 5, to:10, trend:Trend::Decline},
            Stroke{low:2.0, high: 5.0, from:10, to: 15, trend:Trend::Advance},
            Stroke{high:5.0, low:3.5, from: 15, to:20, trend:Trend::Decline},
            Stroke{low:3.5, high:4.5, from: 20, to:25, trend:Trend::Advance},
            Stroke{high:4.5, low:1.5, from: 25, to: 30, trend:Trend::Decline}], 
            vec![
                Pole{index: 0, frac: -1, count: 1, value: 1.0},
                Pole{index: 15, frac: 1, count:1, value: 5.0},
                Pole{index:30, frac: -1, count:1, value: 1.5}
            ]);
    }

    #[test]
    fn split_two() {
        // 图一: 有缺口情况但是线段封闭 406 页
        test_split(vec![
            Stroke{low:1.0, high:3.0, from: 0, to:5, trend:Trend::Advance},
            Stroke{high:3.0, low:2.0, from: 5, to:10, trend:Trend::Decline},
            Stroke{low:2.0, high: 5.0, from:10, to: 15, trend:Trend::Advance},
            Stroke{high:5.0, low:1.5, from: 15, to:20, trend:Trend::Decline},
            Stroke{low:1.5, high:4.5, from: 20, to:25, trend:Trend::Advance},
            Stroke{high:4.5, low:2.5, from: 25, to: 30, trend:Trend::Decline},
            Stroke{low:2.5, high:2.8, from: 30, to:35, trend:Trend::Advance},
            Stroke{high:2.8, low:1.8, from: 35, to: 40, trend:Trend::Decline},
            Stroke{low:1.8, high:2.3, from: 40, to:45, trend:Trend::Advance},
            Stroke{high:2.3, low:0.8, from: 45, to: 50, trend:Trend::Decline}], 
            vec![
                Pole{index: 0, frac: -1, count: 1, value: 1.0},
                Pole{index: 15, frac: 1, count:1, value: 5.0},
                Pole{index: 50, frac: -1, count:1, value: 0.8}
            ]);
    }

    #[test]
    fn split_single() {
        test_split(vec![
            Stroke{low:1.0, high:3.0, from:0, to:5, trend:Trend::Advance},
            Stroke{high:3.0, low:2.6, from:5, to:10, trend:Trend::Decline},
            Stroke{low:2.6, high:5.0, from:10, to:15, trend:Trend::Advance},
            Stroke{high:5.0, low:1.5, from:15, to:20, trend:Trend::Decline},
            Stroke{low:1.5, high:4.5, from:20, to:25, trend:Trend::Advance},
            Stroke{high:4.5, low:2.8, from:25, to:30, trend:Trend::Decline},
            Stroke{low:2.8, high:5.5, from:30, to:35, trend:Trend::Advance}],
        vec![Pole{value:1.0, frac:-1, index:0, count:1},
            Pole{value:5.5, frac:1, index:35, count:1}]);
    }

    #[test]
    fn split_three() {
        test_split(
            vec![
                Stroke{low:1.0, high:3.0, from:0, to:5, trend:Trend::Advance},
                Stroke{high:3.0, low:2.6, from:5, to:10, trend:Trend::Decline},
                Stroke{low:2.6, high:5.0, from:10, to:15, trend:Trend::Advance},
                Stroke{high:5.0, low:1.5, from:15, to:20, trend:Trend::Decline},
                Stroke{low:1.5, high:4.5, from:20, to:25, trend:Trend::Advance},
                Stroke{high:4.5, low:2.8, from:25, to:30, trend:Trend::Decline},
                Stroke{low:2.8, high:4.0, from:30, to:35, trend:Trend::Advance},
                Stroke{high:4.0, low:2.5, from:35, to:40, trend:Trend::Decline},
                Stroke{low:2.5, high:2.9, from:40, to:45, trend:Trend::Advance},
                Stroke{high:2.9, low:2.7, from:45, to:50, trend:Trend::Decline},
                Stroke{low:2.7, high:5.5, from:50, to:55, trend:Trend::Advance}],
            vec![Pole{value:1.0, frac:-1, index:0, count:1},
                Pole{value:5.0, frac:1, index:15, count:1},
                Pole{value:2.5, frac:-1, index:40, count:1},
                Pole{value:5.5, frac:1, index:55, count:1}]
        );
    }

    #[test]
    fn split_four() {
        test_split(
            vec![
                Stroke{low:1.0, high:2.0, from:0, to:5, trend:Trend::Advance},
                Stroke{high:2.0, low:1.5, from:5, to:10, trend:Trend::Decline},
                Stroke{low:1.5, high:5.0, from:10, to:15, trend:Trend::Advance},
                Stroke{high:5.0, low:4.0, from:15, to:20, trend:Trend::Decline},
                Stroke{low:4.0, high:5.5, from:20, to:25, trend:Trend::Advance},
                Stroke{high:5.5, low:3.8, from:25, to:30, trend:Trend::Decline},
                Stroke{low:3.8, high:5.2, from:30, to:35, trend:Trend::Advance},
                Stroke{high:5.2, low:4.5, from:35, to:40, trend:Trend::Decline},
                Stroke{low:4.5, high:5.1, from:40, to:45, trend:Trend::Advance},
                Stroke{high:5.1, low:3.0, from:45, to:50, trend:Trend::Decline}],
            vec![Pole{value:1.0, frac:-1, index:0, count:1},
                Pole{value:5.5, frac:1, index:25, count:1},
                Pole{value:3.0, frac:-1, index:50, count:1}]
        );
    }

    
    #[test]
    fn split_five() {
        test_split(
            vec![
                Stroke{low:1.0, high:2.0, from:0, to:5, trend:Trend::Advance},
                Stroke{high:2.0, low:1.5, from:5, to:10, trend:Trend::Decline},
                Stroke{low:1.5, high:5.0, from:10, to:15, trend:Trend::Advance},
                Stroke{high:5.0, low:4.0, from:15, to:20, trend:Trend::Decline},
                Stroke{low:4.0, high:6., from:20, to:25, trend:Trend::Advance},
                Stroke{high:6., low:4.2, from:25, to:30, trend:Trend::Decline},
                Stroke{low:4.2, high:5.2, from:30, to:35, trend:Trend::Advance},
                Stroke{high:5.2, low:4.5, from:35, to:40, trend:Trend::Decline},
                Stroke{low:4.5, high:5.1, from:40, to:45, trend:Trend::Advance},
                Stroke{high:5.1, low:4.6, from:45, to:50, trend:Trend::Decline},
                Stroke{low:4.6, high:7.0, from:50, to:55, trend:Trend::Advance}],
            vec![Pole{value:1.0, frac:-1, index:0, count:1},
                Pole{value:7.0, frac:1, index:55, count:1}]
        );

    }

    #[test]
    fn sample() {
        let strokes = vec![Stroke { high: 42.85, low: 42.36, from: 0, to: 17, trend: Advance }, Stroke { high: 42.85, low: 42.45, from: 17, to: 36, trend: Decline }, Stroke { high: 42.66, low: 42.45, from: 36, to: 41, trend: Advance }, Stroke { high: 42.66, low: 42.24, from: 41, to: 61, trend: Decline }, Stroke { high: 42.65, low: 42.24, from: 61, to: 81, trend: Advance }, Stroke { high: 42.65, low: 42.02, from: 81, to: 103, trend: Decline }, Stroke { high: 43.0, low: 42.02, from: 103, to: 128, trend: Advance }, Stroke { high: 43.0, low: 42.51, from: 128, to: 140, trend: Decline }, Stroke { high: 43.23, low: 42.51, from: 140, to: 149, trend: Advance }, Stroke { high: 43.23, low: 42.78, from: 149, to: 171, trend: Decline }, Stroke { high: 43.17, low: 42.78, from: 171, to: 190, trend: Advance }, Stroke { high: 43.17, low: 42.8, from: 190, to: 203, trend: Decline }, Stroke { high: 43.18, low: 42.8, from: 203, to: 220, trend: Advance }, Stroke { high: 43.18, low: 42.88, from: 220, to: 229, trend: Decline }, Stroke { high: 43.1, low: 42.88, from: 229, to: 233, trend: Advance }, Stroke { high: 43.1, low: 42.57, from: 233, to: 280, trend: Decline }, Stroke { high: 43.07, low: 42.57, from: 280, to: 307, trend: Advance }, Stroke { high: 43.07, low: 42.72, from: 307, to: 322, trend: Decline }, Stroke { high: 42.88, low: 42.72, from: 322, to: 331, trend: Advance }, Stroke { high: 42.88, low: 41.0, from: 331, to: 357, trend: Decline }, Stroke { high: 41.49, low: 41.0, from: 357, to: 365, trend: Advance }, Stroke { high: 41.49, low: 40.8, from: 365, to: 370, trend: Decline }, Stroke { high: 41.59, low: 40.8, from: 370, to: 398, trend: Advance }, Stroke { high: 41.59, low: 41.21, from: 398, to: 403, trend: Decline }, Stroke { high: 41.52, low: 41.21, from: 403, to: 408, trend: Advance }, Stroke { high: 41.52, low: 41.36, from: 408, to: 417, trend: Decline }];
        test_split(strokes, Vec::new());
    
    }
    #[test]
    fn qk_split_one() {
        test_split(
            vec![
                Stroke{low:1.0, high:2.0, from:0, to:5, trend:Trend::Advance},
                Stroke{high:2.0, low:1.5, from:5, to:10, trend:Trend::Decline},
                Stroke{low:1.5, high:5.0, from:10, to:15, trend:Trend::Advance},
                Stroke{high:5.0, low:4.0, from:15, to:20, trend:Trend::Decline},
                Stroke{low:4.0, high:6., from:20, to:25, trend:Trend::Advance},
                Stroke{high:6., low:5.2, from:25, to:30, trend:Trend::Decline},
                Stroke{low:5.2, high:5.5, from:30, to:35, trend:Trend::Advance},
                Stroke{high:5.5, low:4.5, from:35, to:40, trend:Trend::Decline},
                Stroke{low:4.5, high:8., from:40, to:45, trend:Trend::Advance},
                Stroke{high:8., low:5.8, from:45, to:50, trend:Trend::Decline},
                Stroke{low:5.8, high:7.0, from:50, to:55, trend:Trend::Advance},
                Stroke{high:7., low:5.1, from:55, to:60, trend:Trend::Decline}],
            vec![Pole{value:1.0, frac:-1, index:0, count:1},
                Pole{value:8.0, frac:1, index:45, count:1},
                Pole{value:5.1, frac:-1, index:60, count:2}]
        );
    }

    #[test]
    fn qk_split_two() {
        test_split(
            vec![
                Stroke{low:1.0, high:2.0, from:0, to:5, trend:Trend::Advance},
                Stroke{high:2.0, low:1.5, from:5, to:10, trend:Trend::Decline},
                Stroke{low:1.5, high:5.0, from:10, to:15, trend:Trend::Advance},
                Stroke{high:5.0, low:4.0, from:15, to:20, trend:Trend::Decline},
                Stroke{low:4.0, high:6., from:20, to:25, trend:Trend::Advance},
                Stroke{high:6., low:5.2, from:25, to:30, trend:Trend::Decline},
                Stroke{low:5.2, high:5.5, from:30, to:35, trend:Trend::Advance},
                Stroke{high:5.5, low:4.5, from:35, to:40, trend:Trend::Decline},
                Stroke{low:4.5, high:5.8, from:40, to:45, trend:Trend::Advance},
                Stroke{high:5.8, low:5.3, from:45, to:50, trend:Trend::Decline},
                Stroke{low:5.3, high:7.0, from:50, to:55, trend:Trend::Advance},
                Stroke{high:7., low:4.6, from:55, to:60, trend:Trend::Decline}],
            vec![Pole{value:1.0, frac:-1, index:0, count:1},
                Pole{value:6.0, frac:1, index:25, count:1},
                Pole{value:4.5, frac:-1, index:40, count:1},
                Pole{value:7., frac:1, index:55, count:1}]
        );
    }

    fn test_split(strokes: Vec<Stroke>, poles: Vec<Pole>) {
        let mut handler = Segmenter::new();
        for s in strokes.as_slice() {
            handler.add_stroke(s.clone());
        }

        let result = handler.get_segments();
        println!("poles: {:?}", result);
        let (pivots, bp) = Segmenter::get_pivots(&result, &strokes);
        println!("pivots: {:?}", pivots);
        find_buy_points(&result, &pivots, &strokes);
        assert_eq!(result.len(), poles.len());
        let mut index = 0;
        for pole in result.as_slice() {
            if let Some(seg_pole) = poles.get(index) {
                assert_eq!(*seg_pole, *pole);
            } else {
                assert!(false);
            }
            index += 1;
        }

    }

    fn find_buy_points(poles: &Vec<Pole>, pivots: &Vec<Pivot>, strokes: &Vec<Stroke>) {
        let mut buy1 = Vec::with_capacity(poles.len() / 2);
        let mut z_poles = poles.iter().zip(poles.iter().skip(1));
        let mut z_pivots = pivots.iter().zip(pivots.iter().skip(1));
        while let Some((pp, np)) = z_pivots.next() {
            if pp.trend.advance() && pp.trend == np.trend && pp.lowest > np.highest {
                if let Some(pair) = z_poles.find(|&item| item.0.index < pp.start && item.1.index > np.end) {
                    buy1.push(pair.1.index);
                }
            }
        }

        let mut buy3 = None;
        let mut sell3 = None;
        if let Some(second) = pivots.last() {
            let slen = strokes.len();
            if slen > 3 {
                if let (Some(b1), Some(b2), Some(b3)) = (strokes.get(slen-3), strokes.get(slen- 2), strokes.last()) {
                    if b3.low > second.high {
                        if b1.from == second.end {
                            buy3 = Some(b3.from);
                        } else if b2.from == second.end {
                            buy3 = Some(b3.to);
                        }
                    } else if b2.high < second.low {
                        if b2.from == second.end {
                            sell3 = Some(b2.to);
                        } else if b1.from == second.end {
                            sell3 = Some(b2.from);
                        }
                    }
                }
            }
        }
        println!("buy1: {:?}\r\nbuy3: {:?}\r\nsell3: {:?}", buy1, buy3, sell3);
    }
}