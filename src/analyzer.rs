use crate::judger::{Edge, Vertex};

/*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Current,
    //Higher,
}
*/

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Signal {
    Buy(i32, usize), // 信号，位置
    Sell(i32, usize),
}

/// 交易区间，中枢
#[derive(Debug, Clone)]
pub struct TradingRange {
    //level: Level,
    vertexes: Vec<Vertex>,
}
impl TradingRange {
    fn has_gap(&self, e: Vertex) -> bool {
        match e.edge {
            Edge::Peak => e.value < self.get_support(),
            Edge::Trough => e.value > self.get_resistance(),
        }
    }

    pub fn get_support(&self) -> f32 { 
        if self.vertexes[0].edge == Edge::Peak {
            self.vertexes[1].value.max(self.vertexes[3].value)
        } else {
            self.vertexes[0].value.max(self.vertexes[2].value)
        }
    }

    pub fn get_resistance(&self) -> f32 { 
        if self.vertexes[0].edge == Edge::Peak {
            self.vertexes[0].value.min(self.vertexes[2].value)
        } else {
            self.vertexes[1].value.min(self.vertexes[3].value)
        }
    }

    pub fn start(&self) -> usize {
        self.vertexes[0].index
    }

    // 中枢最后一个点是否延申方向的最高/低点
    pub fn is_end_extended(&self) -> bool {
        let last = self.vertexes[self.vertexes.len() - 1];
        let first = self.vertexes[0];
        if first.edge == last.edge {
            (last.edge == Edge::Peak && self.vertexes.iter().all(|v| v.value <= last.value)) ||
            last.edge == Edge::Trough && self.vertexes.iter().all(|v| v.value >= last.value)
        } else { false }
    }

    pub fn end(&self) -> usize {
        self.vertexes[self.vertexes.len() - 1].index
    }
    
    /// 中枢被第三类买卖点结束，对中枢进行分解为左右两部分
    /// 如果左侧中枢失效则返回 false
    /// 并返回右侧剩余的顶点
    fn finish(&mut self) -> (bool, Option<Vec<Vertex>>) {
        let mut valid = true;
        let leave_at = self.find_leave_at();
        let new_vertexes: Option<Vec<Vertex>>;
        if leave_at == 0 {
            // 反向中枢
            if self.vertexes.len() <= 4 {
                // 中枢失效
                valid = false;
            }
            self.vertexes.remove(0);
            new_vertexes = Some(vec!(self.vertexes.pop().unwrap()));
        } else {
            // 尝试分解中枢
            if leave_at > 3 {
                new_vertexes = Some(self.vertexes.drain(leave_at..).collect());
            } else {
                new_vertexes = Some(self.vertexes.drain(4..).collect());
            }
        }
        (valid, new_vertexes)
    }
    
    /// 扩展中枢分解，分解后左侧中枢不失效
    fn decompose(&mut self) -> Option<Vec<Vertex>> {
        let mut new_vertexes: Option<Vec<Vertex>> = None;
        let leave_at = self.find_leave_at();
        if leave_at == 0 {
            // 反向中枢
            self.vertexes.remove(0);
            new_vertexes = None;
        } else if leave_at > 3 {
            // 尝试分解中枢
            new_vertexes = Some(self.vertexes.drain(leave_at..).collect());
        }
        new_vertexes
    }

    fn find_leave_at(&self) -> usize {
        let mut a = self.vertexes[0];
        let mut leave_at = 0;
        let b = self.vertexes[1];
        for i in 2..self.vertexes.len() { 
            let c = self.vertexes[i];
            if a.edge == c.edge && a.right_included(b, c){ 
                a = c;
                leave_at = i;
            }
        }
        leave_at
    }
    
    fn need_decompose(&mut self) -> bool {
        self.vertexes.len() >= 11
    }

}

impl Vertex {
    // 对于 N 线段，有极点 a,b,c,d,e, 使用极值 b 和 e(self) 判断两个特征笔之间是否有缺口
    // return true if there is a gap, false if bc and be overlap with 
    #[inline]
    fn has_gap(&self, b: f32) -> bool {
        match self.edge {
            Edge::Peak => self.value < b,
            Edge::Trough => self.value > b,
        }
    }

    // 顶点被 a-b 构成的线段左包含，即 c 的值在 a-b 直接
    fn left_included(&self, a: Vertex, b: Vertex) -> bool {
        match self.edge {
            Edge::Peak => self.value <= a.value && self.value > b.value,
            Edge::Trough => self.value >= a.value && self.value < b.value,
        }
    }

    fn right_included(&self, b: Vertex, c: Vertex) -> bool {
        match self.edge {
            Edge::Trough => self.value < b.value && self.value >= c.value,
            Edge::Peak => self.value > b.value && self.value <= c.value,
        }
    }
}

pub enum Status {
    A0,
    Range,
}

/// 走势分析器
pub struct Analyzer;
impl Analyzer {
    pub fn analyze(vertexes: &[Vertex]) -> Vec<TradingRange> {
        let mut trading_ranges: Vec<TradingRange> = Vec::new();
        let mut i = 0;
        let mut status = Status::A0;
        while i < vertexes.len() { 
            match status {
                Status::A0 => { 
                    if (i + 5) > vertexes.len() { 
                        break;
                    }
                    if let [a, b, c, d, e] = vertexes[i..i+5] {
                        if d.has_gap(a.value) || (e.has_gap(b.value) && c.left_included(a, b)) { // 次级别延申
                            i += 1;
                        } else {
                            trading_ranges.push(TradingRange { /* level: Level::Current, */ vertexes: vec![b, c, d, e] });
                            status = Status::Range;
                            i += 5;
                        }
                    }
                },
                Status::Range => { 
                    let e = vertexes[i];
                    if let Some(last_range) = trading_ranges.last_mut() { 
                        if last_range.need_decompose() {
                            let remains = last_range.decompose();
                            if let Some(remains) = remains { 
                                if e.edge != last_range.vertexes[0].edge { 
                                    i -= 1;
                                }
                                status = Status::A0;
                                i -= remains.len();
                                continue;
                            }
                        } 
                        if last_range.has_gap(e) {
                            let (valid, remains) = last_range.finish();
                            if let Some(remains) = remains { 
                                if e.edge != last_range.vertexes[0].edge { 
                                    i -= 1;
                                }
                                if !valid { 
                                    trading_ranges.pop();
                                }
                                status = Status::A0;
                                i -= remains.len();
                                continue;
                            } else if !valid { 
                                trading_ranges.pop();
                                status = Status::A0;
                                i -= 1;
                                continue;
                            }
                        } else {
                            last_range.vertexes.push(e);
                        }
                    }
                    i += 1;
                },
            }
        }
        trading_ranges.retain(|tr| tr.vertexes.len() >= 4);
        trading_ranges
    }
    pub fn signals(trading_ranges: &[TradingRange], vertexes: &[Vertex]) -> Vec<Signal> {
        let mut signals = Vec::new();
        let mut vi = 0;
        let len = vertexes.len();
        let mut last_tr: Option<&TradingRange> = None;
        for tr in trading_ranges {
            let end = tr.end();
            while vi < len && vertexes[vi].index <= end { 
                vi += 1;
            }
            if let Some(last_tr) = last_tr { 
                if vi >= len { 
                    if tr.vertexes.len() > 3 && tr.is_end_extended() {
                        vi -= 1;
                    } else {
                        break;
                    }
                }
                let v = vertexes[vi];
                let edge = tr.vertexes[0].edge;
                if ((edge == Edge::Trough 
                    && tr.get_resistance() < last_tr.get_support())
                    || (edge == Edge::Peak && tr.get_support() > last_tr.get_resistance())
                ) && v.edge == edge { 
                    if edge == Edge::Peak { 
                        signals.push(Signal::Sell(1, v.index));
                        if vi + 2 < len { 
                            let v = vertexes[vi+2];
                            signals.push(Signal::Sell(2, v.index));
                        }
                    } else {
                        signals.push(Signal::Buy(1, v.index));
                        if vi + 2 < len { 
                            let v = vertexes[vi+2];
                            signals.push(Signal::Buy(2, v.index));
                        }
                    }
                }
            }
            vi += 1; // 第三类买卖点检查
            if vi >= len {
                break;
            }
            let v = vertexes[vi];
            let support = tr.get_support();
            let resistance = tr.get_resistance();
            if v.value < support && v.edge == Edge::Peak {
                signals.push(Signal::Sell(3, v.index));
            } else if v.value > resistance && v.edge == Edge::Trough {
                signals.push(Signal::Buy(3, v.index));
            }            
            last_tr = Some(tr);
        }
        signals
    }
}

#[cfg(test)]
mod tests {
    use crate::judger::Vertex;

    use super::*;
    use rstest::rstest;

    impl Edge {
        pub(crate) fn oppsite(&self) -> Edge {
            match self {
                Edge::Peak => Edge::Trough,
                Edge::Trough => Edge::Peak,
            }
        }
    }

    impl Vertex {
        pub fn to_vertexs(vertex_values: &[f32], indexes: Option<&[usize]>) -> Vec<Vertex> {
            assert!(indexes.is_none() || indexes.unwrap().len() == vertex_values.len(), "indexes length must be equal to vertex_values length");
            let mut vertexs = Vec::with_capacity(vertex_values.len());
            let mut edge = Edge::Trough;
            for i in 1..=vertex_values.len() {
                let i = i-1;
                if i == 0 && vertex_values[i] > vertex_values[i + 1] {
                    edge = Edge::Peak;
                }
                let index = if let Some(indexes) = indexes {
                    indexes[i]
                } else {
                    i * crate::judger::BI_COUNT
                };
                vertexs.push(Vertex::new(index, edge, vertex_values[i]));
                edge = edge.oppsite();
            }
            vertexs
        }
    }

    #[rstest]
    #[case::one_extended_with_3b(&[15., 10., 12., 9., 11., 8., 11.5, 10.2, 11.3, 9.5, 14., 13., 14., 10.], Some(vec![10.0, 12.0, 9.0, 11.0, -1.0, 11.5, 10.2, 11.3, 9.5, -1.0, 14.0, 13.0, 14.0, 10.0]))]
    #[case::one_extended_with_3b_new_one(&[15., 10., 12., 9., 11., 8., 11.5, 10.2, 11.3, 9.5, 14., 13., 14., 12.5, 15.0], Some(vec![10.0, 12.0, 9.0, 11.0, -1.0, 11.5, 10.2, 11.3, 9.5, -1.0, 14.0, 13.0, 14.0, 12.5, 15.0]))]
    #[case::up_with_3s(&[10.0, 12.0, 11.5, 12.5, 11.0, 11.6, 10.8, 11.1, 10.5], Some(vec![12.0, 11.5, 12.5, 11.0]))]
    #[case::none2(&[11., 9., 9.5, 8., 10.9, 10., 12., 11.1, 12., 10., 11., 9., 9.5, 8., 8.9, 8.5, 9.5, 9., 10.], Some(vec![9.0, 9.5, 8.0, 10.9, -1.0, 12.0, 11.1, 12.0, 10.0]))]
    #[case::one_with_3s(&[10., 11., 9., 9.5, 8., 8.5, 8.2, 8.8, 8.3, 8.7, 6., 7., 5.], Some(vec![8.0, 8.5, 8.2, 8.8, 8.3, 8.7]))]
    #[case::two_down_ranges(&[15., 10., 11., 9., 10.5, 8., 8.5, 7., 8.8, 5., 5.5, 4.], Some(vec![10.0, 11.0, 9.0, 10.5, -1.0, 8.0, 8.5, 7.0, 8.8]))]
    #[case::two_down_ranges2(&[32.56, 56.23, 39.24, 41.85, 38.19, 42.90, 39.01, 41.52, 32.70, 34.77, 32.18, 35.50, 30.80, 33.68, 30.44], Some(vec![39.24, 41.85, 38.19, 42.90, 39.01, 41.52, -1.0, 32.70, 34.77, 32.18, 35.50, 30.80, 33.68, 30.44]))]
    #[case::none(&[10.0, 11., 9., 9.5, 8., 8.5, 8.2, 9.8, 8.8, 10.], None)]
    #[case::one(&[10., 11., 9., 9.5, 8., 8.5, 8.2, 8.8, 8.3, 8.7], Some(vec![8.0, 8.5, 8.2, 8.8, 8.3, 8.7]))]
    #[case::one_extended(&[15., 10., 12., 9., 11., 8., 11.5, 10.2, 11.3, 9.5, 10.8], Some(vec![10.0, 12.0, 9.0, 11.0, 8.0, 11.5, 10.2, 11.3, 9.5, 10.8]))]
    fn test_analyzer(#[case] vertex_values: &[f32], #[case] expected: Option<Vec<f32>>) {
        let vertexes = Vertex::to_vertexs(vertex_values, None);

        let trading_ranges = Analyzer::analyze(&vertexes);
        if expected.is_none() {
            assert_eq!(trading_ranges.len(), 0);
            return;
        }
        let mut result = Vec::new();
        trading_ranges.iter().for_each(|tr| {
            if !result.is_empty() {
                result.push(-1.0); // 分隔符
            }
            tr.vertexes.iter().for_each(|v| {
                result.push(v.value);
            })
        });
        // println!("{:?}", trading_ranges);
        assert_eq!(result, expected.unwrap());
    } 

    #[rstest]
    #[case::sample(&[2.2, -0.59, 2.35, 0.86, 17.98, 3.47, 9.06, 6.14, 9.6, 6.41, 15.07, 8.61, 18.37, 6.45, 9.68, 6.87, 14.02, 11.14, 33.72, 24.3, 83.16, 40.37, 84.36, 57.03, 73.58, 20.99, 35.07, 25.68, 65.9], None)]
    #[case::sell1(&[1.0, 2.0, 1.5, 2.5, 2.0, 4.0, 2.6, 4.5, 3.9, 5.0, 4.6, 4.7, 3.9], Some(vec![Signal::Buy(3, 30), Signal::Sell(1, 45), Signal::Sell(2, 55), Signal::Buy(3,50)]))]
    #[case::buy1_last(&[4.0, 1.5, 2.5, 2.0, 3.0, 1.0, 1.2, 0.9, 1.1, 0.8], Some(vec![Signal::Sell(3,30),Signal::Buy(1, 45)]))]
    #[case::buy1_reverse_last(&[1.0, 4.0, 1.5, 2.5, 2.0, 3.0, 1.0, 1.2, 0.9, 1.1, 0.8], Some(vec![Signal::Sell(3,35),Signal::Buy(1, 50)]))]
    #[case::buy1(&[4.0, 1.5, 2.5, 2.0, 3.0, 1.0, 1.2, 0.9, 1.1, 0.8, 0.85, 0.81], Some(vec![Signal::Sell(3,30),Signal::Buy(1, 45), Signal::Buy(2, 55), Signal::Sell(3, 50)]))]
    #[case::buy3(&[1.0, 2.0, 1.5, 2.5, 2.0, 4.0, 2.6], Some(vec![Signal::Buy(3, 30)]))]
    #[case::sell3(&[4.0, 1.5, 2.5, 2.0, 3.0, 1.0, 1.2], Some(vec![Signal::Sell(3, 30)]))]
    #[case::sell1_last(&[1.0, 2.0, 1.5, 2.5, 2.0, 4.0, 2.6, 4.5, 3.9, 5.0], Some(vec![Signal::Buy(3, 30), Signal::Sell(1, 45)]))]
    fn test_analyzer_signals(#[case] vertex_values: &[f32], #[case] expected: Option<Vec<Signal>>) {
        let vertexes = Vertex::to_vertexs(vertex_values, None);
        let trading_ranges = Analyzer::analyze(&vertexes);
        let signals = Analyzer::signals(&trading_ranges, &vertexes);
        assert_eq!(signals, expected.unwrap());
    }
}