use std::collections::HashSet;

use crate::judger::{Edge, Vertex};

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
}

// 一个 N 需要 a, b, c, d 四个极值，最多需要三个 N
const N_POLE_SIZE: usize = 4;
const N_FIRST: usize = 1; // 第一个 N
const N_SECOND: usize = 2;
const N_THIRD: usize = 3;
// 分段器
pub struct Segmenter {
    /* 极值，四个极值构成一个 N, 连续三个 N 需要最多 10 个极值
     */
    vertexes: Vec<Vertex>, 
    state: State, // 最新分段状态，对应图谱中的编号

    segmented_index: Vec<usize>, // 已分段的索引位置
    merged_feature_vertexs: Vec<Vertex>, // 存放被合并的反向特征序列极值，生成新段时清空
}
impl Segmenter {
    pub fn new() -> Self {
        Segmenter::with_capacity(10)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self { vertexes: Vec::with_capacity(capacity), state: State::None, segmented_index: Vec::new(), merged_feature_vertexs: Vec::new() }
    }

    // 森林漫步，寻找分段点
    pub fn step(&mut self, vertex: &Vertex) {
        self.vertexes.push(*vertex);
        match self.state {
            State::None => {
                if self.vertexes.len() >= N_POLE_SIZE {
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
            self.merged_feature_vertexs.clear();
        }
    }
    
    // 相邻两个同向笔有缺口，跳过 skip 个同向笔 Vertex 数量, 跳过一个同向笔则 skip=2
    fn has_feature_gap(&self, skip: usize) -> bool {
        if let (Some(first), Some(last)) = (self.last_vertex(N_POLE_SIZE + skip), self.vertexes.last()) {
            match first.edge {
                Edge::Peak => last.value > first.value,
                Edge::Trough => last.value < first.value,
            }
        } else { false }
    }

    #[inline]
    fn last_vertex(&self, to_end: usize) -> Option<&Vertex> {
        self.vertexes.get(self.vertexes.len() - to_end)
    }
    
    // 第一个 N 方向顺势突破
    fn forward(&self) -> bool {
        if let (Some(prev), Some(last)) = (self.vertexes.get(N_POLE_SIZE - 1), self.last_vertex(1)) {
            match prev.edge {
                Edge::Peak => last.value > prev.value,
                Edge::Trough => last.value < prev.value,
            }
        } else { false }
    }

    // 搜索第一段起点，向上段还是向下段，从第一个点开始还是从第二个点开始
    // 如果搜索到返回 true, 并对起点进行必要的修正
    fn search_start_point(&mut self) -> bool {
        if self.has_feature_gap(0) {
            self.vertexes.remove(0);
            false
        } else if let (Some(first), Some(second), Some(last)) = (self.last_vertex(N_POLE_SIZE), self.last_vertex(N_POLE_SIZE - 1), self.last_vertex(1)) {
            let found = match first.edge {
                Edge::Peak => last.value < second.value,
                Edge::Trough => last.value > second.value,
            };
            if found { self.vertexes.drain(0..self.vertexes.len() - N_POLE_SIZE); }
            found
        } else { false }
    }
    
    // 第一个 N 逆势突破
    fn backward(&self) -> bool {
        if let (Some(first), Some(last)) = (self.vertexes.first(), self.vertexes.last()) {
            match first.edge {
                Edge::Peak => last.value >= first.value,
                Edge::Trough => last.value <= first.value,
            }
        } else { false }
    }

    // 合并第 n 个 N 为一笔, n 从 1 开始
    #[inline]
    fn merge_n(&mut self, n: usize) {
        let from = (n - 1) * (N_POLE_SIZE - 1) + 1;
        let to = from + 2;
        let drain = self.vertexes.drain(from..to);
        if n > 1 { self.merged_feature_vertexs.extend(drain ) };
    }

    pub fn indexes(&self) -> HashSet<usize> {
        let mut result: HashSet<usize> = HashSet::with_capacity(self.segmented_index.len() + 2);
        result.extend(self.segmented_index.as_slice());
        result
    }
    
    // 最后 Point 落在左笔中, 被包含
    fn converge(&self) -> bool {
        if let (Some(prev), Some(last)) = (self.last_vertex(3), self.last_vertex(1)) {
            match prev.edge {
                Edge::Peak => prev.value > last.value,
                Edge::Trough => prev.value < last.value,
            }
        } else { false }
    }
    
    // 将第一个 N 入段并移除
    fn push_segment(&mut self) {
        if let Some(vertex) = self.vertexes.first() {
            if self.segmented_index.is_empty() {
                self.segmented_index.push(vertex.index); // 段开始索引，避免首尾重复
            }
            self.vertexes.drain(..N_POLE_SIZE-1);
            if let Some(vertex) = self.vertexes.first() {
                self.segmented_index.push(vertex.index); // 段结束索引
            }
        }
        self.merged_feature_vertexs.clear();
    }
    
    // 返回中阴阶段被合并的 vertexs
    //pub fn intermediate(&self) -> Vec<Vertex> {
    //    self.merged_feature_vertexs.clone()
    //}
    
    //最后虚段(未完成)的临时终结点
    pub fn tip(&self) -> usize {
        match self.state {
            State::None => 0,
            _ => self.vertexes.get(N_POLE_SIZE - 1).unwrap().index
        }
    }
    
    // 特征序列具有反向顶底分型
    fn check_feature_reverse(&mut self) -> bool {
        if self.merged_feature_vertexs.len() >= 4 {
            let mut vertexs = self.merged_feature_vertexs.clone();
            vertexs.extend_from_slice(&self.vertexes[self.vertexes.len() - 3 ..]);
            vertexs.sort_by_key(|p| p.index);
            let mut f = Segmenter::new();
            for p in &vertexs {
                f.step(p);
            }
            let indexes = f.indexes();
            if !indexes.is_empty() {
                let max = indexes.iter().max().unwrap();
                self.segmented_index.push(vertexs.first().unwrap().index);
                self.segmented_index.push(*max);
                //self.segmented_index.extend(&indexes);
                self.vertexes.clear();
                self.vertexes.extend(&f.vertexes);
                self.merged_feature_vertexs.clear();
                self.merged_feature_vertexs.extend(&f.merged_feature_vertexs);
                self.state = f.state;
                return true;
            }
        }
        false
    }
        
}

#[cfg(test)]
mod tests {
    use crate::judger::BI_COUNT;

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(State::None, &[], &[1.0, 3.0, 2.0])]
    #[case(State::None, &[],&[1.0, 3.0, 2.0, 2.5])]
    #[case(State::S0, &[0, 3],&[1.0, 3.0, 2.0, 5.0])]
    #[case(State::S0, &[1, 4],&[6., 1.0, 3.0, 2.0, 5.])]
    #[case(State::S0, &[2, 5],&[1.0, 3.0, 2.0, 2.8, 2.5, 5.])]
    #[case(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0])]
    #[case(State::S2, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5])]
    #[case(State::S0, &[0, 5],&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0])]
    #[case(State::S12, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5])]
    #[case(State::S0, &[0, 5],&[1.0, 3.0, 2.0, 5.0, 2.5, 6.0])]
    #[case(State::S22, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5])]
    #[case(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 4.2])]
    #[case(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5])]
    #[case(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5])]
    #[case(State::S0, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 0.5])]
    #[case(State::S1, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 4.2])]
    #[case(State::S2, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 2.8])]
    #[case(State::S0, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 2.5, 4.5, 2.2])]
    #[case(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8])]
    #[case(State::S1222, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2])]
    #[case(State::S12, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 4.6])]
    #[case(State::S0, &[0, 7],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 6.])]
    #[case(State::S1231,&[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8])]
    #[case(State::S1232, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2])]
    #[case(State::S22, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.8])]
    #[case(State::S0, &[0, 7],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 6.0])]
    #[case(State::S12211, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6])]
    #[case(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.4])]
    #[case(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 2.9])]
    #[case(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 0.9])]
    #[case(State::S12221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9])]
    #[case(State::S122, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.2])]
    #[case(State::S123, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 2.9])]
    #[case(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 0.9])]
    #[case(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 1.8])]
    #[case(State::S12312, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2])]
    #[case(State::S0, &[0, 3, 8],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 2.0])]
    #[case(State::S12322, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5])]
    #[case(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 3.7])]
    #[case(State::S122, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 3.9])]
    #[case(State::S123, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 4.3])]
    #[case(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.5, 3.8, 3.6, 6.])]
    #[case(State::S1221, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 3.95])]
    #[case(State::S1222, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 4.1])]
    #[case(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 3.8, 4.2, 3.9, 4.6])]
    #[case(State::S1231, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 3.6])]
    #[case(State::S122, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 3.9])]
    #[case(State::S123, &[0, 3, 6],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 4.8])]
    #[case(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 3.8, 3.2, 6.0])]
    #[case(State::S1231, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 3.8])]
    #[case(State::S1232, &[0, 3],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 4.1])]
    #[case(State::S0, &[0, 3, 6, 9],&[1.0, 3.0, 2.0, 5.0, 4.0, 4.5, 2.5, 4.2, 3.5, 4.6])]
    #[case(State::S0, &[0, 5, 14, 19],&[1.0, 3.0, 2.0, 5.0, 4.0, 6.0, 4.5, 5.5, 3.2, 4.8, 1.5, 6.2, 1.2, 3.5, 0.5, 1.3, 0.8, 5.6, 3.0, 7.])]
    fn test_segmenter(#[case] expected: State, #[case] segmented: &[usize], #[case] vertex_values: &[f32]) {
        let mut segmenter = Segmenter::new();
        let vertexs = Vertex::to_vertexs(vertex_values, None);
        for vertex in vertexs {
            segmenter.step(&vertex);
        }
        let indexes = segmenter.indexes();
        let segmented = if segmented.len() < 3 { &[] } else { &segmented[0..&segmented.len()-1] };
        assert_eq!(segmented.iter().map(|s| *s * BI_COUNT).collect::<std::collections::HashSet<usize>>(), indexes);
        assert_eq!(expected, segmenter.state);
    }

        
}