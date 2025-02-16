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
    pub end_index: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BiMode {
    pub tuibi: bool, // 推笔，顶底分型不允许包含
    pub quekou: bool, // 缺口突破直接成笔
}
impl BiMode {
    pub(crate) fn new(mode: i32) -> Self {
        let tuibi = mode / 1000 & 2 == 2;
        let quekou = mode / 1000 & 4 == 4;
        Self { tuibi, quekou }
    }
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
            end_index: index + init_count - 1,
        }
    }
    pub fn sample(index: usize, high: f32, low: f32, count: usize, up: bool) -> Self {
        Self {
            index,
            high,
            low,
            up,
            count,
            end_index: index + count - 1,
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
        self.end_index
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
    None = 555,
    S0 = 60,
    S1 = 61,
    S11 = 611,
    S12 = 612,
    S121 = 6121,
    S122 = 6122,
    S1221 = 61221,
    S12211 = 612211,
    S122111 = 6122111,
    S122112 = 6122112,
    S122113 = 6122113,
    S122114 = 6122114,
    S12212 = 612212,
    S12213 = 612213,
    S12214 = 612214,
    S1222 = 61222,
    S12221 = 612221,
    S122211 = 6122211,
    S122212 = 6122212,
    S122213 = 6122213,
    S122214 = 6122214,
    S12222 = 612222,
    S12223 = 612223,
    S12224 = 612224,
    S1223 = 61223,
    S1224 = 61224,
    S123 = 6123,
    S1231 = 61231,
    S12311 = 612311,
    S12312 = 612312,
    S123121 = 6123121,
    S123122 = 6123122,
    S123123 = 6123123,
    S123124 = 6123124,
    S1232 = 61232,
    S12321 = 612321,
    S12322 = 612322,
    S123221 = 6123221,
    S123222 = 6123222,
    S123223 = 6123223,
    S1233 = 61233,
    S1234 = 61234,
    S124 = 6124,
    S2 = 62,
    S21 = 621,
    S22 = 622,
    S221 = 6221,
    S222 = 6222,
    S223 = 6223,
}

#[derive(Debug)]
pub struct Pivot {
    pub poles: Vec<Pole>,
    pub extended: bool,
    contains_3bs: bool, // 是否包含第三类买卖点，两个中枢重叠排除第三类买卖点的重叠
}

impl Pivot {
    pub fn start(&self) -> usize {
        self.poles[0].index
    }

    pub fn end(&self) -> usize {
        self.poles[self.poles.len() - 1].index
    }

    // 回落笔的 pole 极值，是否在中枢内
    pub fn converge(&self, d: &Pole, e: &Pole) -> bool {
        match e.edge {
            Edge::TROUGH => e.value <= self.high() && d.value >= self.low(),
            Edge::PEAK => e.value >= self.low() && d.value <= self.high(),
        }
    }

    // 中枢 ZG = min(g1, g2)，只需取前面四个极值进行判定
    pub fn high(&self) -> f32 {
        self.poles.iter().take(4).filter(|p| p.edge == Edge::PEAK).map(|p| p.value).min_by(f32::total_cmp).unwrap()
    }
    // 中枢 ZD = max(d1, d2)，只需取前面四个极值进行判定
    pub fn low(&self) -> f32 {
        self.poles.iter().take(4).filter(|p| p.edge == Edge::TROUGH).map(|p| p.value).max_by(f32::total_cmp).unwrap()
    }
    
    // 中枢扩张判定
    fn expand(&self, next: &Pivot) -> bool {
        next.high() < self.low() && next.highest(true) >= self.lowest(false) 
        || next.low() > self.high() && next.lowest(true) <= self.highest(false)
    }
    
    fn merge(&mut self, next: Pivot) {
        self.poles.extend(next.poles);
    }
    
    // 中枢 GG，判定中枢扩张时排除第三类买卖点
    fn highest(&self, skip_3bs: bool) -> f32 {
        self.poles.iter().skip(if skip_3bs && self.contains_3bs {2} else {0}).filter(|p| p.edge == Edge::PEAK).map(|p| p.value).max_by(f32::total_cmp).unwrap()
    }

    // 中枢 DD，判定中枢扩张时排除第三类买卖点
    fn lowest(&self, skip_3bs: bool) -> f32 {
        self.poles.iter().skip(if skip_3bs && self.contains_3bs {2} else {0}).filter(|p| p.edge == Edge::TROUGH).map(|p| p.value).min_by(f32::total_cmp).unwrap()
    }

    fn last_edge(&self) -> Edge {
        self.poles[self.poles.len() - 1].edge
    }
    
    fn check_extended(&mut self) {
        if !self.extended && self.poles.len() >= N_POLE_SIZE * 3 - 2 {
            self.extended = true;
        }
    }
}
// 一个 N 需要 a, b, c, d 四个极值，最多需要三个 N
const N_POLE_SIZE: usize = 4;
const N_FIRST: usize = 1; // 第一个 N
const N_SECOND: usize = 2;
const N_THIRD: usize = 3;
// 分段器
pub struct Forest {
    /* 极值，四个极值构成一个 N, 连续三个 N 需要最多 10 个极值
     */
    poles: Vec<Pole>, 
    state: State, // 最新分段状态，对应图谱中的编号

    with_entry: bool, // 是否设置进入段，如果设置中枢从下一段开始，否则从当前段开始
    with_signals: bool, // 是否输出买卖点
    
    segmented_index: Vec<usize>, // 已分段的索引位置
    merged_feature_poles: Vec<Pole>, // 存放被合并的反向特征序列极值，生成新段时清空
}
impl Forest {
    pub fn new() -> Self {
        Self { poles: Vec::with_capacity(10), state: State::None, segmented_index: Vec::new(), merged_feature_poles: Vec::new(), with_entry: true, with_signals: true }
    }
    
    fn with(with_entry: bool, with_signals: bool) -> Self {
        Self { poles: Vec::with_capacity(10), state: State::None, segmented_index: Vec::new(), merged_feature_poles: Vec::new(), with_entry, with_signals }
    }

    pub fn state(&self) -> State {
        self.state
    }

    // 森林漫步，寻找分段点
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
                } else { 
                    self.state = State::S12; 
                    // 如果已经多次合并且出现反向顶底分型，则线段结束于反向顶底位置
                    self.check_feature_reverse();
                }
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
        if self.state == State::S0 { // 顺向创新高或新低后，将中阴极点去除
            self.merged_feature_poles.clear();
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
    
    // 第一个 N 方向顺势突破
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
        result.extend(self.segmented_index.as_slice());
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
            if self.segmented_index.is_empty() {
                self.segmented_index.push(pole.index); // 段开始索引，避免首尾重复
            }
            self.poles.drain(..N_POLE_SIZE-1);
            if let Some(pole) = self.poles.first() {
                self.segmented_index.push(pole.index); // 段结束索引
            }
        }
        self.merged_feature_poles.clear();
    }
    
    // 按照分段进行内部中枢划分
    pub fn pivots(&self, poles: &Vec<Pole>) -> Vec<Pivot> {
        let mut pivots: Vec<Pivot> = Vec::new();
        let mut last_segmented_index = std::usize::MAX;
        // 根据是否设置进入段，设定至少需要的极点数量
        let min_pole_size = N_POLE_SIZE + if self.with_entry { 1 } else { 0 };
        for index in 0..poles.len() {
            let pole = poles.get(index).unwrap();
            if last_segmented_index == std::usize::MAX {
                last_segmented_index = index;
            } else if pole.segmented || index == poles.len() - 1 {
                if (index - last_segmented_index + 1) >= min_pole_size {
                    pivots.extend(self.find_pivots(poles, last_segmented_index, index));
                }
                last_segmented_index = index;
            }
        }
        pivots
    }

    // 将段内的极点进行中枢划分
    fn find_pivots(&self, poles: &[Pole], start: usize, end: usize) -> Vec<Pivot> {
        let mut pivots : Vec<Pivot> = Vec::new();
        // 如果设置进入段，那中枢区间划分从第二笔开始，否则直接从第一笔开始
        let mut i = start + 1;
        while (i + 3) <= end {
            if let (Some(b), Some(c), Some(d), Some(e)) 
            = (poles.get(i), poles.get(i + 1), poles.get(i + 2), poles.get(i + 3)) {
                if !e.has_gap(b.value) {
                    let new_pivot = if let Some(last) = pivots.last_mut() {
                        if last.converge(d, e) { // 中枢延申
                            last.poles.push(*d);
                            last.poles.push(*e);
                            last.check_extended();
                            false
                        } else if b.index > last.end() { // 新中枢
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    };
                    if new_pivot {
                        // 第三类买卖点可以跌破或升破中枢的高高或低低点，但是中枢扩张时后续的极点不允许
                        // 记录下第三类买卖点帮助后面区别对待
                        let contains_3bs = !pivots.is_empty() && poles.get(i-1).unwrap().index == pivots.last().unwrap().end();
                        // if contains_3bs && !self.with_entry {
                        //     pivots.last_mut().unwrap().poles.pop();
                        // }
                        pivots.push(Pivot { poles: vec![*b, *c, *d, *e], extended: false, contains_3bs });
                    }
                } 
            }
            i += 2;
        }
        // 中枢扩张为更高级别中枢检查
        let mut i = 0;
        while pivots.len() > 1 && i <= pivots.len() - 2 {
            if let (Some(prev), Some(next)) = (pivots.get(i), pivots.get(i+1)) {
                if prev.expand(next) {
                    pivots.get_mut(i).unwrap().extended = true;
                    pivots.get_mut(i + 1).unwrap().extended = true;
                }
                i += 1;
            }
        }
        pivots
    }

    /* 根据极值和中枢寻找买卖点：
     * 1. 一类买卖点，在两个同向中枢后的分段点 segmented 处设置
     * 2. 二类买卖点，在一类买卖点之后的首个和分段点同向点处，根据最后一个中枢位置判定
     * 3. 三类买卖点，根据最后一个中枢对拉回中枢的极值点进行判定
     * 4. 二三买卖点可能重合
     * 5. 考虑类二买卖点?
    */
    fn signals(&self, poles: &[Pole], pivots: &Vec<Pivot>) -> Option<HashMap<usize, Signal>> {
        if !self.with_signals {
            return None
        }
        let mut pivot_index = 0;
        let mut segment_start = usize::MAX;
        let mut segment_start_idx = usize::MAX;
        let mut signals = Signals::new();
        let last_idx = poles.len() - 1;
        for (idx, pole) in poles.iter().enumerate() {
            let segmented = pole.segmented || idx == last_idx;
            if segmented {
                if segment_start != usize::MAX {
                    // 如果线段没有中枢但是笔数大于三笔，也设置一类买卖点（小级别）
                    // 线段终结，如果是趋势终结，设置一类买卖点, 暂不考虑中枢扩展
                    let mut found = (idx - segment_start_idx) > N_POLE_SIZE; // 线段笔数5以上
                    let mut last_pivot = None;
                    if pivot_index > 0 {
                        let prev = pivots.get(pivot_index - 1).unwrap();
                        if prev.end() > segment_start {
                            found = false;
                            if pivot_index > 1 {
                                let former = pivots.get(pivot_index - 2).unwrap();
                                if former.last_edge() == prev.last_edge() && former.start() > segment_start && prev.end() < pole.index {
                                    last_pivot = Some(prev);
                                    found = true;
                                }
                            }
                        }
                    }
                    if found {
                        signals.insert(pole.index, pole.signal1());
                        
                        if let Some(pole) = poles.get(idx + 2) {
                            // 分段拐头后的第一个反向笔，检查二类买卖点，前一段可能没有中枢 
                            if let Some(pivot) = last_pivot {
                                signals.insert(pole.index, pole.signal2_by(pivot));
                            } else if let (Some(former), Some(prev)) = (poles.get(idx - 2), poles.get(idx - 1)) {
                                // 本级别一类买卖点
                                let mock_pivot = Pivot { poles: vec![*former, *prev], extended: false, contains_3bs: false };
                                signals.insert(pole.index, pole.signal2_by(&mock_pivot));
                            }
                        }
                    }
                }
                segment_start = pole.index;
                segment_start_idx = idx;
            }

            if let Some(pivot) = pivots.get(pivot_index) {
                if pole.index == pivot.end() {
                    if let Some(next) = poles.get(idx + 1) {
                        if let Some(signal) = next.signal3_by(pivot) {
                            signals.insert(next.index, signal);
                        }
                    }
                    if let Some(next) = poles.get(idx + 2) {
                        if let Some(signal) = next.signal3_by(pivot) {
                            signals.insert(next.index, signal);
                        }
                    }              
                    pivot_index += 1;
                }
            }
        }
        Some(signals)
    }
    
    // 返回中阴阶段被合并的 poles
    fn intermediate(&self) -> Vec<Pole> {
        self.merged_feature_poles.clone()
    }
    
    //最后虚段(未完成)的临时终结点
    fn tip(&self) -> usize {
        match self.state {
            State::None => 0,
            _ => self.poles.get(N_POLE_SIZE - 1).unwrap().index
        }
    }
    
    // 特征序列具有反向顶底分型
    fn check_feature_reverse(&mut self) -> bool {
        if self.merged_feature_poles.len() >= 4 {
            let mut poles = self.merged_feature_poles.clone();
            poles.extend_from_slice(&self.poles[self.poles.len() - 3 ..]);
            poles.sort_by_key(|p| p.index);
            let mut f = Forest::new();
            for p in poles {
                f.step(&p);
            }
            let indexes = f.indexes();
            if indexes.len() > 0 {
                self.segmented_index.extend(&indexes);
                self.poles.clear();
                self.poles.extend(&f.poles);
                self.merged_feature_poles.clear();
                self.merged_feature_poles.extend(&f.merged_feature_poles);
                self.state = f.state;
                return true;
            }
        }
        false
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
    pub segmented: bool, // 段极点
    pub trended: bool, // 趋势极点
    pub trend_upgraded: bool, // 更大级别趋势极点
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
            trended: false,
            trend_upgraded: false,
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
    
    fn signal2_by(&self, pivot:&Pivot) -> Signal {
        match self.edge {
            Edge::PEAK =>  if self.value < pivot.low() { Signal::SELL23 } else { Signal::SELL2 },
            Edge::TROUGH => if self.value > pivot.high() { Signal::BUY23 } else { Signal::BUY2 },
        }
    }
    fn signal2(&self) -> Signal {
        match self.edge {
            Edge::PEAK => Signal::SELL2,
            Edge::TROUGH => Signal::BUY2,
        }
    }

}

// 分笔器
#[derive(Debug)]
pub struct Tracer {
    strokes: Vec<Stroke>,
    bi_mode: BiMode,
}

impl Tracer {
    pub fn new(capacity: usize) -> Self {
        Self {
            strokes: Vec::with_capacity(capacity),
            bi_mode: BiMode { tuibi: false, quekou: false},
        }
    }

    pub fn with_bi_mode(capacity: usize, bi_mode: BiMode) -> Self {
        Self { strokes: Vec::with_capacity(capacity), bi_mode }
    }

    pub fn with(strokes: Vec<Stroke>) -> Self {
        Self { strokes, bi_mode: BiMode { tuibi: false, quekou: false} }
    }

    // 根据K线及前后关系逐一扫描进行粉笔
    // merged 为 true 表示K线包含
    // gap 为 true 表示有缺口
    // up 为 true 表示与前K线相比的方向向上
    fn update(&mut self, index: usize, high: f32, low: f32, up: bool, merged: bool, gap: bool) {
        if 0 == index {
            self.strokes.push(Stroke::new(index, high, low, up));
            return;
        }
        let last_stroke = self.strokes.last().unwrap();
        
        if merged || up == last_stroke.up {
            let leap = if !merged {
                let prev_stroke = if self.strokes.len() > 2 { self.strokes.get(self.strokes.len() - 2) } else {None};
                if let Some(prev_stroke) = prev_stroke {
                    // 缺口突破
                    (up && low < prev_stroke.high && high > prev_stroke.high && (gap /*|| (prev_stroke.done() && !last_stroke.done())*/)) // 只保留缺口突破处理
                    || (!up && high > prev_stroke.low && low < prev_stroke.low && (gap /* || (prev_stroke.done() && !last_stroke.done())*/))
                } else { false }
            } else { false };
            let last_stroke = self.strokes.last_mut().unwrap();
            if !merged {
                if leap && self.bi_mode.quekou {
                    last_stroke.count = STEPS;
                } else {
                    last_stroke.count += 1;
                }
            }
            last_stroke.end_index = index;
            if up && last_stroke.high < high {
                last_stroke.high = high;
            } else if !up && last_stroke.low > low {
                last_stroke.low = low;
            }
        } else {
            let leap = (up && !last_stroke.up && gap && high > last_stroke.high) 
            || (!up && last_stroke.up && gap && low < last_stroke.low);
            let high = if up { high } else { last_stroke.high };
            let low = if up { last_stroke.low } else { low };
            if index == 1 {
                let last_stroke = self.strokes.last_mut().unwrap();
                last_stroke.low = low;
                last_stroke.count += 1;
                last_stroke.end_index = index;
            } else {
                let mut stroke = Stroke::new(index - 1, high, low, up);
                if leap && self.bi_mode.quekou {
                    // 缺口反包
                    stroke.count = STEPS;
                }
                self.strokes.push(stroke);
            }
        }

        while self.strokes.len() > 2 {
            // 对已经生成的不满足条件的笔进行合并处理
            let last_stroke = self.strokes.last().unwrap();
            let mut former_index = self.strokes.len() - 3;
            let last_end_index = last_stroke.end_index;
            let last_start_index = last_stroke.index;
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
                count += former.count + previous.count - 2;
                if !previous.done() || !former.done() {
                    if right_include {
                        // former 笔被 previous 笔包含了，将 former+previous 笔合并到 former 前的笔
                        if former_index > 0 {
                            let value = if up { previous.low } else { previous.high };
                            former_index -= 1;
                            let former = self.strokes.get_mut(former_index).unwrap();
                            count += former.count;
                            if up {
                                former.low = value;
                            } else {
                                former.high = value;
                            }
                            former.end_index = last_start_index;
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
                        former.end_index = last_end_index;
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

    // 获取分笔完成后的所有笔极点
    pub fn poles(&self) -> Vec<Pole> {
        let valid_strokes = self.strokes.iter()
        .skip_while(|p| !p.done())
        .take_while(|p| p.done()).collect::<Vec<_>>();
        let mut result: Vec<Pole> = valid_strokes
            .iter()
            .map(|stroke| Pole::new(stroke.index, Edge::from(stroke.up), stroke.start(), false))
            .collect();
        if let Some(last) = valid_strokes.last() {
            if last.count >= STEPS {
                result.push(Pole::new(last.end_index, Edge::from(!last.up), last.stop(), false));
            }
        }
        
        result
    }
}

// 市场行情数据
pub struct Market<'a> {
    pub high: &'a [f32],
    pub low: &'a [f32],
    pub len: usize,

    pub merged_index: Vec<i8>, // 合并关系
    pub tracer: Tracer,
}

// 根据合并关系获取前笔高低值
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
        Market::with_bi_mode(len, high, low, BiMode::default())
    }
    
    pub fn with_bi_mode(len: usize, high: *mut f32, low: *mut f32, bi_mode: BiMode) -> Self {
        let mut result = Self {
            high: unsafe { std::slice::from_raw_parts_mut(high, len) },
            low: unsafe { std::slice::from_raw_parts_mut(low, len) },
            len,
            merged_index: vec![-1; len],
            tracer: Tracer::with_bi_mode(len / 7, bi_mode),
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

    // 使用分笔器合并K线并寻找笔顶底
    fn merge(&mut self) {
        let mut up = true;
        self.tracer
            .update(0, self.high[0], self.low[0], true, false, false);
        for i in 1..self.len {
            let mut curr_high = self.high[i];
            let mut curr_low = self.low[i];
            let prev_high = self.get_prev_high(i - 1, up);
            let prev_low = self.get_prev_low(i - 1, up);
            let mut merged = false;
            let mut gap = false;
            match (
                curr_high.total_cmp(&prev_high),
                curr_low.total_cmp(&prev_low),
            ) {
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                    // 上升
                    up = true;
                    gap = curr_low > prev_high;
                }
                (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => {
                    // 下降
                    up = false;
                    gap = curr_high < prev_low;
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
            self.tracer.update(i, curr_high, curr_low, up, merged, gap);
        }
    }

    pub fn zigzag_with_signals(&self) -> ZigzagResult {
        self.zigzag_with_flag(false, PivotMode::BI)
    }

    pub fn zigzag(&self, mode: PivotMode, with_signals: bool, with_entry: bool) -> ZigzagResult {
        zigzag(self.tracer.poles(), mode, with_signals, with_entry)
    }
    pub fn zigzag_with_flag(&self, skip_signal: bool, mode: PivotMode) -> ZigzagResult {
        self.zigzag(mode, !skip_signal, true)
    }
}

#[derive(Debug)]
pub struct ZigzagResult {
    pub spins: Option<Vec<Pole>>,
    pub bi_zigzag: Option<Zigzag>,
    pub duan_zigzag: Option<Zigzag>,
    pub trend_zigzag: Option<Zigzag>,
}

impl ZigzagResult {
    pub fn new() -> Self {
        Self { spins: None, bi_zigzag: None, duan_zigzag: None, trend_zigzag: None }
    }
}

// 中枢划分级别，按笔、段或走势进行划分
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PivotMode {
    BI,
    DUAN,
    TREND,
}

impl PivotMode {
    pub fn new(mode: i32) -> Self {
        match mode % 100 / 10 {
            1 => PivotMode::DUAN,
            2 => PivotMode::TREND,
            _ => PivotMode::BI,
        }
    }

    pub fn is_bi(&self) -> bool {
        match self {
            PivotMode::BI => true,
            _ => false,
        }
    }

    pub fn is_duan(&self) -> bool {
        match self {
            PivotMode::DUAN => true,
            _ => false,
        }
    }

    pub fn is_trend(&self) -> bool {
        match self {
            PivotMode::TREND => true,
            _ => false,
        }
    }
}

// 根据小级别的极值信息和中枢模式进行中枢划分和买卖信号识别
// 根据需要从笔中枢逐步升级至段中枢和走势中枢
fn zigzag(mut spins: Vec<Pole>, mode: PivotMode, with_signals: bool, with_entry: bool) -> ZigzagResult {
    let mut zz = ZigzagResult::new();
    let (forest, pivots, signals, intermediate) = zigzag_internal(with_signals, with_entry, &mut spins);
    zz.bi_zigzag = Some(Zigzag { intermediate, pivots, signals, state: forest.state(), tip: forest.tip() });
    if mode != PivotMode::BI {
        let tip = forest.tip();
        let mut segments = spins.iter().filter(|pole| pole.segmented || (tip > 0 && pole.index == tip)).cloned().collect::<Vec<Pole>>();
        segments.iter_mut().for_each(|p| p.segmented = false);
        let (forest, pivots, signals, intermediate) = zigzag_internal(with_signals, with_entry, &mut segments);
        zz.duan_zigzag = Some(Zigzag { intermediate, pivots, signals, state: forest.state(), tip: forest.tip() });
        let segmented_indexes: HashSet<usize> = segments.iter().filter_map(|p| if p.segmented { Some(p.index) } else { None }).collect();
        spins.iter_mut().for_each(|p| {
            if segmented_indexes.contains(&p.index) { 
                p.trended = true 
            }
            if p.index == tip {
                p.segmented = true;
            }
        });
        if mode != PivotMode::DUAN {
            let tip = forest.tip();
            segments = segments.iter().filter(|pole| pole.segmented || (tip > 0 && pole.index == tip)).cloned().collect::<Vec<Pole>>();
            segments.iter_mut().for_each(|p| p.segmented = false);
            let (forest, pivots, signals, intermediate) = zigzag_internal(with_signals, with_entry, &mut segments);
            let segmented_indexes: HashSet<usize> = segments.iter().filter_map(|p| if p.segmented { Some(p.index) } else { None }).collect();
            spins.iter_mut().for_each(|p| {
                if segmented_indexes.contains(&p.index) { 
                    p.trend_upgraded = true 
                }
                if p.index == tip {
                    p.trended = true;
                }
            });
            zz.trend_zigzag = Some(Zigzag { intermediate, pivots, signals, state: forest.state(), tip: forest.tip() });
        }
    }
    zz.spins = Some(spins);
    zz
}

fn zigzag_internal(with_signals: bool, with_entry: bool, spins: &mut Vec<Pole>) -> (Forest, Vec<Pivot>, Option<HashMap<usize, Signal>>, Vec<Pole>) {
    let mut forest = Forest::with(with_entry, with_signals);
    for pole in &*spins {
        forest.step(pole);
    }
    let indexes = forest.indexes();
    let last_segmented_index = indexes.iter().max().unwrap_or_else(|| &0);
    for pole in spins.as_mut_slice() {
        if indexes.contains(&pole.index) {
            (*pole).segmented = true;
        }
    }
    let pivots= forest.pivots(&*spins);
    let signals = forest.signals(&*spins, &pivots);
    
    let intermediate = forest.intermediate();
    // 处于中阴状态的极点，将其移除
    spins.retain(|&pole | {
        // let retain = 
        (*last_segmented_index > 0 && pole.index <= *last_segmented_index)
         || intermediate.iter().find(|&p| p.index == pole.index ).is_none()
        // println!("{:?} {:?}", pole, retain);
        // retain
    });
    (forest, pivots, signals, intermediate)
}

pub type Signals = HashMap<usize, Signal>;
#[derive(Debug)]
pub struct Zigzag {
    pub intermediate: Vec<Pole>,
    pub pivots: Vec<Pivot>,
    pub signals: Option<Signals>,
    pub state: State,
    pub tip: usize, // 最后段的临时终结点
}