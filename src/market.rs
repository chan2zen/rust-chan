#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

pub const STEPS: usize = 5;
#[derive(Debug)]
pub struct Stroke {
    pub index: usize,
    pub high: f32,
    pub low: f32,
    pub up: bool,
    pub count: usize,
    pub days: usize,
    pub segmented: bool,
}

impl Stroke {
    pub fn new(index: usize, high: f32, low: f32, up: bool) -> Self {
        let init_count = if index == 0 { 1 } else { 2 };
        Self {
            index,
            high,
            low,
            up,
            count: init_count,
            days: init_count,
            segmented: false,
        }
    }
    pub fn sample(index: usize, high: f32, low: f32, count: usize, up: bool) -> Self {
        Self {
            index,
            high,
            low,
            up,
            count,
            days: count,
            segmented: false,
        }
    }

    fn value(&self) -> f32 {
        if self.up {
            self.high
        } else {
            self.low
        }
    }

    fn start(&self) -> f32 {
        if self.up {
            self.low
        } else {
            self.high
        }
    }

    fn stop(&self) -> f32 {
        if self.up {
            self.high
        } else {
            self.low
        }
    }

    #[inline]
    fn done(&self) -> bool {
        self.count >= STEPS
    }
    
    fn end_index(&self) -> usize {
        self.index + self.days - 1
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Edge {
    PEAK = 1,
    TROUGH = -1,
}

impl From<bool> for Edge {
    fn from(up: bool) -> Self {
        if up {
            Edge::TROUGH
        } else {
            Edge::PEAK
        }
    }
}

impl Edge {
    pub(crate) fn oppsite(&self) -> Edge {
        match self {
            Edge::PEAK => Edge::TROUGH,
            Edge::TROUGH => Edge::PEAK,
        }
    }

}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    None,
    S0,
    S1,
    S11,
    S12,
    S121,
    S122,
    S1221,
    S12211,
    S122111,
    S122112,
    S122113,
    S122114,
    S12212,
    S12213,
    S12214,
    S1222,
    S12221,
    S122211,
    S122212,
    S122213,
    S122214,
    S12222,
    S12223,
    S12224,
    S1223,
    S1224,
    S123,
    S1231,
    S12311,
    S12312,
    S123121,
    S123122,
    S123123,
    S123124,
    S1232,
    S12321,
    S12322,
    S123221,
    S123222,
    S123223,
    S1233,
    S1234,
    S124,
    S2,
    S21,
    S22,
    S221,
    S222,
    S223,
}

#[derive(Debug)]
pub struct Pivot {
    pub poles: Vec<Pole>,
}

impl Pivot {
    pub fn start(&self) -> usize {
        self.poles[0].index
    }

    pub fn end(&self) -> usize {
        self.poles[self.poles.len() - 1].index
    }

    // 回落笔的 pole 极值，是否在中枢内
    pub fn converge(&self, pole: &Pole) -> bool {
        match pole.edge {
            Edge::TROUGH => pole.value <= self.high(),
            Edge::PEAK => pole.value >= self.low(),
        }
    }
    pub fn high(&self) -> f32 {
        self.poles.iter().filter(|p| p.edge == Edge::PEAK).map(|p| p.value).min_by(f32::total_cmp).unwrap()
    }

    pub fn low(&self) -> f32 {
        self.poles.iter().filter(|p| p.edge == Edge::TROUGH).map(|p| p.value).max_by(f32::total_cmp).unwrap()
    }
    
    fn overlap(&self, next: &Pivot) -> bool {
        next.high() < self.low() && next.highest() >= self.lowest() 
        || next.low() > self.high() && next.lowest() <= self.highest()
    }
    
    fn merge(&mut self, next: Pivot) {
        self.poles.extend(next.poles);
    }
    
    fn highest(&self) -> f32 {
        self.poles.iter().filter(|p| p.edge == Edge::PEAK).map(|p| p.value).max_by(f32::total_cmp).unwrap()

    }
    
    fn lowest(&self) -> f32 {
        self.poles.iter().filter(|p| p.edge == Edge::TROUGH).map(|p| p.value).min_by(f32::total_cmp).unwrap()

    }
}
// 一个 N 需要 a, b, c, d 四个极值，最多需要三个 N
const N_POLE_SIZE: usize = 4;
const N_FIRST: usize = 1; // 第一个 N
const N_SECOND: usize = 2;
const N_THIRD: usize = 3;
pub struct Forest {
    /* 极值，四个极值构成一个 N, 连续三个 N 需要最多 10 个极值
     */
    poles: Vec<Pole>, 
    state: State,
    
    segmented_index: Vec<usize>,
    merged_feature_poles: Vec<Pole>, // 存放被合并的反向特征序列极值，生成新段时清空
}
impl Forest {
    pub fn new() -> Self {
        Self { poles: Vec::with_capacity(10), state: State::None, segmented_index: Vec::new(), merged_feature_poles: Vec::new() }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn step(&mut self, pole: &Pole) {
        self.poles.push(pole.clone());
        match self.state {
            State::None => {
                if self.poles.len() >= N_POLE_SIZE {
                    // 寻找 N 再开始
                    if self.search_start_point() {
                        self.state = State::S0;
                    }
                }
            },
            State::S0 => {
                if self.has_feature_gap(0) {
                    self.state = State::S1;
                } else {
                    self.state = State::S2;
                }
            }, 
            State::S1 => {
                if self.forward() {
                    self.state = State::S11;
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                } else { self.state = State::S12; }
            },
            State::S2 => {
                if self.forward() {
                    self.state = State::S21;
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                } else {
                    self.state = State::S22;
                }
            }
            State::S12 => {
                if self.converge() {
                    self.state = State::S121;
                    self.merge_n(N_SECOND);
                    self.state = State::S1;
                } else if self.has_feature_gap(2) {
                    self.state = State::S122;
                } else if self.backward() {
                    self.state = State::S124;
                    self.push_segment();
                    self.state = State::S0;
                } else {
                    self.state = State::S123;
                }
            },
            State::S22 => {
                if self.converge() {
                    if self.has_feature_gap(2) {
                        self.state = State::S221;
                        self.state = State::S1;
                    } else {
                        self.state = State::S222;
                        self.state = State::S2;
                    }
                    self.merge_n(N_SECOND);
                } else {
                    self.push_segment();
                    self.state = State::S0;
                }
            },
            State::S122 => {
                if self.has_feature_gap(0) {
                    self.state = State::S1221;
                } else if self.converge() {
                    self.state = State::S1222;
                } else {
                    self.merge_n(N_SECOND);
                    if self.forward() {
                        self.state = State::S1224;
                        self.merge_n(N_FIRST);
                        self.state = State::S0;
                    } else {
                        self.state = State::S1223;
                        self.state = State::S12;
                    }
                }
            },
            State::S123 => {
                if self.has_feature_gap(0) {
                    self.state = State::S1231;
                } else if self.converge() {
                    self.state = State::S1232;
                } else {
                    self.merge_n(N_SECOND);
                    if self.forward() {
                        self.state = State::S1234;
                        self.merge_n(N_FIRST);
                        self.state = State::S0;
                    } else {
                        self.state = State::S1233;
                        self.state = State::S22;
                    }
                }
            },
            State::S1221 => {
                if self.converge() {
                    self.state = State::S12211;
                } else if self.has_feature_gap(4) {
                    self.state = State::S12212;
                    self.merge_n(N_SECOND);
                    self.state = State::S122;
                } else if self.backward() {
                    self.state = State::S12214;
                    self.push_segment();
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                } else {
                    self.state = State::S12213;
                    self.merge_n(N_SECOND);
                    self.state = State::S123;
                }
            },
            State::S1231 => {
                if self.converge() {
                    self.state = State::S12312;
                } else {
                    self.state = State::S12311;
                    self.push_segment();
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                }
            },
            State::S1232 => {
                if self.converge() {
                    self.state = State::S12322;
                } else {
                    self.state = State::S12321;
                    self.push_segment();
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                }
            },
            State::S12211 => {
                if self.converge() {
                    self.state = State::S122111;
                    self.merge_n(N_THIRD);
                    self.state = State::S1221;
                } else {
                    self.push_segment();
                    if self.has_feature_gap(2) {
                        self.state = State::S122112;
                        self.state = State::S122;
                    } else if self.backward() {
                        self.state = State::S122114;
                        self.push_segment();
                        self.state = State::S0;
                    } else {
                        self.state = State::S122113;
                        self.state = State::S123;
                    }
                }
            },
            State::S1222 => {
                if self.backward() {
                    self.state = State::S12224;
                    self.push_segment();
                    self.merge_n(N_FIRST);
                    self.state = State::S0;
                } else if self.has_feature_gap(4) {
                    if self.converge() {
                        self.state = State::S12221;
                    } else {
                        self.state = State::S12222;
                        self.state = State::S122;
                        self.merge_n(N_SECOND);
                    }
                } else {
                    self.state = State::S12223;
                    self.merge_n(N_SECOND);
                    self.state = State::S123;
                }
            },
            State::S12221 => {
                if self.converge() {
                    if self.has_feature_gap(2) {
                        self.state = State::S122211;
                        self.state = State::S1221;
                    } else {
                        self.state = State::S122212;
                        self.state = State::S1222;
                    }
                    self.merge_n(N_THIRD);
                } else {
                    self.state = State::S122213;
                    self.push_segment();
                    self.push_segment();
                    self.state = State::S0;
                }
            },
            State::S12312 => {
                if self.forward() {
                    self.state = State::S123124;
                    self.push_segment();
                    self.push_segment();
                    self.state = State::S0;
                } else if self.converge() {
                    self.state = State::S123121;
                    self.merge_n(N_THIRD);
                    self.state = State::S1231;
                } else if self.has_feature_gap(2) {
                    self.state = State::S123122;
                    self.push_segment();
                    self.state = State::S122;
                } else {
                    self.state = State::S123123;
                    self.push_segment();
                    self.state = State::S123;
                }
            },
            State::S12322 => {
                if self.converge() {
                    if self.has_feature_gap(2) {
                        self.state = State::S123221;
                        self.state = State::S1231;
                    } else {
                        self.state = State::S123222;
                        self.state = State::S1232;
                    }
                    self.merge_n(N_THIRD);
                } else {
                    self.state = State::S123223;
                    self.push_segment();
                    self.push_segment();
                    self.state = State::S0;
                }
            },
            _ => {},
        }
    }
    
    // 相邻两个同向笔有缺口，跳过 skip 个同向笔 Pole 数量, 跳过一个同向笔则 skip=2
    fn has_feature_gap(&self, skip: usize) -> bool {
        if let (Some(first), Some(last)) = (self.last_pole(N_POLE_SIZE + skip), self.poles.last()) {
            match first.edge {
                Edge::PEAK => last.value > first.value,
                Edge::TROUGH => last.value < first.value,
            }
        } else { false }
    }

    #[inline]
    fn last_pole(&self, to_end: usize) -> Option<&Pole> {
        self.poles.get(self.poles.len() - to_end)
    }
    
    // 第一个 N 顺势突破
    fn forward(&self) -> bool {
        if let (Some(prev), Some(last)) = (self.poles.get(N_POLE_SIZE - 1), self.last_pole(1)) {
            match prev.edge {
                Edge::PEAK => last.value > prev.value,
                Edge::TROUGH => last.value < prev.value,
            }
        } else { false }
    }

    // 搜索第一段起点，向上段还是向下段，从第一个点开始还是从第二个点开始
    // 如果搜索到返回 true, 并对起点进行必要的修正
    fn search_start_point(&mut self) -> bool {
        if self.has_feature_gap(0) {
            self.poles.remove(0);
            false
        } else if let (Some(first), Some(second), Some(last)) = (self.last_pole(N_POLE_SIZE), self.last_pole(N_POLE_SIZE - 1), self.last_pole(1)) {
            let found = match first.edge {
                Edge::PEAK => last.value < second.value,
                Edge::TROUGH => last.value > second.value,
            };
            if found { self.poles.drain(0..self.poles.len() - N_POLE_SIZE); }
            found
        } else { false }
    }
    
    // 第一个 N 逆势突破
    fn backward(&self) -> bool {
        if let (Some(first), Some(last)) = (self.poles.first(), self.poles.last()) {
            match first.edge {
                Edge::PEAK => last.value >= first.value,
                Edge::TROUGH => last.value <= first.value,
            }
        } else { false }
    }

    // 合并第 n 个 N 为一笔, n 从 1 开始
    #[inline]
    fn merge_n(&mut self, n: usize) {
        let from = (n - 1) * (N_POLE_SIZE - 1) + 1;
        let to = from + 2;
        let drain = self.poles.drain(from..to);
        if n > 1 { self.merged_feature_poles.extend(drain ) };
    }

    pub fn indexes(&self) -> HashSet<usize> {
        let mut result: HashSet<usize> = HashSet::with_capacity(self.segmented_index.len() + 2);
        for idx in self.segmented_index.iter(){
            result.insert(*idx);
        }
        // if self.state != State::None { // 中阴段，不添加
        //     result.insert(self.poles.first().unwrap().index);
        //     result.insert(self.poles.get(N_POLE_SIZE - 1).unwrap().index);
        // }
        result
    }
    
    // 最后 Point 落在左笔中, 被包含
    fn converge(&self) -> bool {
        if let (Some(prev), Some(last)) = (self.last_pole(3), self.last_pole(1)) {
            match prev.edge {
                Edge::PEAK => prev.value > last.value,
                Edge::TROUGH => prev.value < last.value,
            }
        } else { false }
    }
    
    // 将第一个 N 入段并移除
    fn push_segment(&mut self) {
        if let Some(pole) = self.poles.first() {
            self.segmented_index.push(pole.index); // 段开始索引
            self.poles.drain(..N_POLE_SIZE-1);
            if let Some(pole) = self.poles.first() {
                self.segmented_index.push(pole.index); // 段结束索引
            }
        }
        self.merged_feature_poles.clear();
    }
    
    pub fn pivots(&self, poles: &Vec<Pole>) -> (HashMap<usize, Signal>, Vec<Pivot>) {
        let mut result: Vec<Pivot> = Vec::new();
        let mut last_segmented_index = std::usize::MAX;
        let mut signals = HashMap::new();
        for index in 0..poles.len() {
            let pole = poles.get(index).unwrap();
            if pole.segmented || index == poles.len() - 1 {
                if last_segmented_index != std::usize::MAX {
                    if (index - last_segmented_index) > N_POLE_SIZE {
                        let (inner_signals, pivots) = self.find_pivots(poles, last_segmented_index, index);
                        signals.extend(inner_signals);
                        result.extend(pivots);
                    }
                }
                last_segmented_index = index;
            }
        }
        
        (signals, result)
    }
    
    fn find_pivots(&self, poles: &[Pole], last_segmented_index: usize, index: usize) -> (HashMap<usize, Signal>, Vec<Pivot>) {
        let mut pivots : Vec<Pivot> = Vec::new();
        let mut i = last_segmented_index + 1;
        let mut signals = HashMap::new();
        while (i + 3) <= index {
            if let (Some(b), Some(c), Some(d), Some(e)) 
            = (poles.get(i), poles.get(i + 1), poles.get(i + 2), poles.get(i + 3)) {
                if !e.has_gap(b.value) {
                    let new_pivot = if let Some(last) = pivots.last_mut() {
                        if last.converge(e) { // 中枢延申
                            last.poles.push(*d);
                            last.poles.push(*e);
                            false
                        } else if b.index > last.end() { // 新中枢
                            true
                        } else { // 三类买卖点
                            signals.insert(e.index, e.signal3());
                            false
                        }
                    } else {
                        true
                    };
                    if new_pivot {
                        pivots.push(Pivot { poles: vec![*b, *c, *d, *e] });
                    }
                } else if pivots.len() > 0 {
                    let pivot = pivots.last().unwrap();
                    if c.index == pivot.end() {
                        signals.insert(e.index, e.signal3());
                    }

                }
            }
            i += 2;
        }
        // 中枢扩展、扩张, todo!()
        let mut i = 0;
        while pivots.len() > 1 && i <= pivots.len() - 2 {
            if let (Some(prev), Some(next)) = (pivots.get(i), pivots.get(i+1)) {
                if prev.overlap(next) {
                    let next = pivots.remove(i + 1);
                    pivots.get_mut(i).unwrap().merge(next);
                } else { i += 1; }
            }
        }
        if pivots.len() > 1 {
            let pole = poles.get(index).unwrap();
            signals.insert(pole.index, pole.signal1());
            if (index + 2) < poles.len() {
                let pivot = pivots.last().unwrap();
                let pole = poles.get(index + 2).unwrap();
                signals.insert(pole.index, pole.signal2(pivot));
            }
        } 
        if !pivots.is_empty() && (index + 1) < poles.len() {
            let pivot = pivots.last().unwrap();
            let pole = poles.get(index + 1).unwrap();

            if let Some(signal) = pole.signal3_by(pivot){
                signals.insert(pole.index, signal);
            }
        }
        (signals, pivots)
    }
    
    // 返回中阴阶段被合并的 poles
    fn intermediate(&self) -> &[Pole] {
        &self.merged_feature_poles
    }

}
#[derive(Debug, PartialEq, Eq,Clone, Copy)]
pub enum Signal {
    BUY1 = 1,
    SELL1 = -1,
    BUY2 = 2,
    BUY22 = 22,
    SELL22 = -22,
    SELL2 = -2,
    BUY3 = 3,
    SELL3 = -3,
    BUY23 = 23,
    SELL23 = -23,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Pole {
    pub index: usize,
    pub edge: Edge,
    pub value: f32,
    pub segmented: bool,
}

impl Pole {
    pub fn simple(index: usize, edge: Edge) -> Self {
        Self::new(index, edge, 0., false)
    }
    pub fn new(index: usize, edge: Edge, value: f32, segmented: bool) -> Self {
        Self {
            index,
            edge,
            value,
            segmented,
        }
    }

    // 对于 N 线段，有极点 a,b,c,d,e, 使用极值 b 和 e(self) 判断两个特征笔之间是否有缺口
    // return true if there is a gap, false if bc and be overlap with 
    #[inline]
    fn has_gap(&self, b: f32) -> bool {
        match self.edge {
            Edge::PEAK => self.value < b,
            Edge::TROUGH => self.value > b,
        }
    }

    fn signal3(&self) -> Signal {
        match self.edge {
            Edge::PEAK => Signal::SELL3,
            Edge::TROUGH => Signal::BUY3,
        }
    }

    fn signal3_by(&self, pivot:&Pivot) -> Option<Signal> {
        match self.edge {
            Edge::PEAK => if self.value < pivot.low() { Some(Signal::SELL3) } else { None },
            Edge::TROUGH =>  if self.value > pivot.high() { Some(Signal::BUY3) } else { None },
        }
    }
    
    fn signal1(&self) -> Signal {
        match self.edge {
            Edge::PEAK =>  Signal::SELL1 ,
            Edge::TROUGH =>  Signal::BUY1,
        }
    }
    
    fn signal2(&self, pivot:&Pivot) -> Signal {
        match self.edge {
            Edge::PEAK =>  if self.value < pivot.low() { Signal::SELL23 } else { Signal::SELL2 },
            Edge::TROUGH => if self.value > pivot.high() { Signal::BUY23 } else { Signal::BUY2 },
        }
    }

}

#[derive(Debug)]
pub struct Tracer {
    strokes: Vec<Stroke>,
}

impl Tracer {
    pub fn new(capacity: usize) -> Self {
        Self {
            strokes: Vec::with_capacity(capacity),
        }
    }

    pub fn with(strokes: Vec<Stroke>) -> Self {
        Self { strokes }
    }

    fn update(&mut self, index: usize, high: f32, low: f32, up: bool, merged: bool) {
        if 0 == index {
            self.strokes.push(Stroke::new(index, high, low, up));
            return;
        }
        let last_stroke = self.strokes.last_mut().unwrap();

        if merged || up == last_stroke.up {
            last_stroke.days += 1;
            if !merged {
                last_stroke.count += 1;
            }
            if up && last_stroke.high < high {
                last_stroke.high = high;
            } else if !up && last_stroke.low > low {
                last_stroke.low = low;
            }
        } else {
            let high = if up { high } else { last_stroke.high };
            let low = if up { last_stroke.low } else { low };
            if index == 1 {
                self.strokes.pop();
            }
            self.strokes.push(Stroke::new(index - 1, high, low, up));
        }

        while self.strokes.len() > 2 {
            let last_stroke = self.strokes.last().unwrap();
            let mut former_index = self.strokes.len() - 3;
            let mut days = last_stroke.days;
            let mut count = last_stroke.count;
            let high = last_stroke.high;
            let low = last_stroke.low;
            loop {
                let former = self.strokes.get(former_index).unwrap();
                let previous = self.strokes.get(former_index + 1).unwrap();
                let broken = (up && last_stroke.high >= former.high)
                    || (!up && last_stroke.low <= former.low);
                if !broken {
                    // 如果没有顺向突破则无需检查
                    return;
                }
                let right_include = up && previous.low <= former.low || !up && previous.high >= former.high;
                days = days + former.days + previous.days - 2;
                count += former.count + previous.count - 2;
                if !previous.done() || !former.done() {
                    if right_include {
                        // former 笔被 previous 笔包含了，将 former+previous 笔合并到 former 前的笔
                        if former_index > 0 {
                            let value = if up { previous.low } else { previous.high };
                            former_index -= 1;
                            let former = self.strokes.get_mut(former_index).unwrap();
                            days += former.days - 1;
                            count += former.count - 1;
                            if up {
                                former.low = value;
                            } else {
                                former.high = value;
                            }
                            former.days = days;
                            former.count = count;
                            self.strokes.drain(former_index + 1..former_index + 3);
                            break;
                        } else {
                            self.strokes.drain(former_index..former_index + 2);
                            return;
                        }
                    } else {
                        let former = self.strokes.get_mut(former_index).unwrap();
                        if up {
                            former.high = high;
                        } else {
                            former.low = low;
                        }
                        former.days = days;
                        former.count = count;
                        self.strokes.drain(former_index + 1..);
                        break;
                    }
                } else {
                    if former_index > 1 {
                        former_index -= 2;
                    } else {
                        return;
                    }
                }
            }
        }
    }
    pub fn poles(&self, last_index: usize) -> Vec<Pole> {
        let skip_first = !self.strokes.first().unwrap().done();
        let mut result: Vec<Pole> = self
            .strokes
            .iter()
            .skip(if skip_first { 1 } else { 0 })
            .map(|stroke| Pole::new(stroke.index, Edge::from(stroke.up), stroke.start(), false))
            .collect();
        let last = self.strokes.last().unwrap();
        if last.count >= STEPS {
            result.push(Pole::new(last_index, Edge::from(!last.up), last.stop(), false));
        }
        result
    }
}

pub struct Market<'a> {
    pub high: &'a [f32],
    pub low: &'a [f32],
    pub len: usize,

    pub merged_index: Vec<i8>,
    pub tracer: Tracer,
}

#[inline]
fn get_prev_value(values: &[f32], index: &Vec<i8>, i: usize, up: bool) -> f32 {
    let mut value = values[i];
    let mut last_mi = index[i];
    for j in (0..=i).rev() {
        if index[j] < 0 || index[j] > last_mi {
            break;
        }
        let curr_value = values[j];
        if (curr_value > value && up) || (curr_value < value && !up) {
            value = curr_value;
        }
        last_mi = index[j];
    }
    value
}

impl Market<'_> {
    pub fn new(len: usize, high: *mut f32, low: *mut f32) -> Self {
        let mut result = Self {
            high: unsafe { std::slice::from_raw_parts_mut(high, len) },
            low: unsafe { std::slice::from_raw_parts_mut(low, len) },
            len,
            merged_index: vec![-1; len],
            tracer: Tracer::new(len / 7),
        };

        result.merge();
        result
    }

    pub fn strokes(&self) -> &Vec<Stroke> {
        &self.tracer.strokes
    }

    fn get_prev_high(&self, i: usize, up: bool) -> f32 {
        get_prev_value(self.high, &self.merged_index, i, up)
    }

    fn get_prev_low(&self, i: usize, up: bool) -> f32 {
        get_prev_value(self.low, &self.merged_index, i, up)
    }

    // 合并K线并寻找笔顶底
    fn merge(&mut self) {
        let mut up = true;
        self.tracer
            .update(0, self.high[0], self.low[0], true, false);
        for i in 1..self.len {
            let mut curr_high = self.high[i];
            let mut curr_low = self.low[i];
            let prev_high = self.get_prev_high(i - 1, up);
            let prev_low = self.get_prev_low(i - 1, up);
            let mut merged = false;
            match (
                curr_high.total_cmp(&prev_high),
                curr_low.total_cmp(&prev_low),
            ) {
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                    // 上升
                    up = true;
                }
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => {
                    // 下降
                    up = false;
                }
                _ => {
                    // 包含
                    merged = true;
                    if self.merged_index[i - 1] < 0 {
                        self.merged_index[i - 1] += 1;
                    }
                    self.merged_index[i] = self.merged_index[i - 1] + 1;
                    curr_high = if up {
                        curr_high.max(prev_high)
                    } else {
                        curr_high.min(prev_high)
                    };
                    curr_low = if up {
                        curr_low.max(prev_low)
                    } else {
                        curr_low.min(prev_low)
                    };
                }
            }
            self.tracer.update(i, curr_high, curr_low, up, merged);
        }
    }

    pub fn zigzag(&self) -> (Vec<Pole>, HashMap<usize,Signal>, Vec<Pivot>) {
        let mut spins = self.tracer.poles(self.len - 1);
        let mut forest = Forest::new();
        for pole in &spins {
            forest.step(pole);
        }
        let indexes = forest.indexes();
        let last_segmented_index = indexes.iter().max().unwrap_or_else(|| &0);
        let intermediate = forest.intermediate(); // 处于中阴状态的极点
        spins.retain(|&pole | {
            (*last_segmented_index > 0 && pole.index <= *last_segmented_index)
             || intermediate.iter().find(|&p| p.index == pole.index ).is_none()
        });
        for pole in spins.as_mut_slice() {
            if indexes.contains(&pole.index) {
                (*pole).segmented = true;
            }
        }
        let pivots = forest.pivots(&spins);
        (spins, pivots.0, pivots.1)
    }
}
