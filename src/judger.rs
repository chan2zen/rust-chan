use indexmap::IndexMap;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Edge { 
    Peak,
    Trough,
}
impl Edge {
    
    pub(crate) fn as_frac(&self) -> f32 {
        match self {
            Edge::Peak => 1.0,
            Edge::Trough => -1.0,
        }
    }
}

// 更安全、可返回错误的转换 (推荐用于外部输入)
impl TryFrom<i32> for Edge {
    type Error = String; // 错误类型可以根据需要定义

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Edge::Peak),
            -1 => Ok(Edge::Trough),
            _ => Err(format!("数值 '{}' 不是有效的Edge标识。请使用 1 (Peak) 或 -1 (Trough)。", value)),
        }
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Stroke { 
    pub from: usize,
    pub to: usize,
    pub count: usize,
    pub up: bool,
    pub high: f32,
    pub low: f32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vertex { 
    pub index: usize,
    pub value: f32,
    pub edge: Edge,
    pub count: usize,
}
impl Vertex {
    pub(crate) fn new(index: usize, edge: Edge, value: f32) -> Self {
        Self { index, value, edge, count: BI_COUNT }
    }

}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Include {
    Left,
    Right,
    Equal,
}

pub const BI_COUNT: usize = 5;

impl Stroke { 
    pub fn new(from: usize, to: usize, count: usize, high: f32, low: f32, up: bool) -> Self { 
        Stroke { from, to, count, up, high, low }
    }

    pub fn is_done(&self) -> bool { 
        self.count >= BI_COUNT
    }

    pub fn mark_done(&mut self) { 
        self.count = BI_COUNT;
    }

    pub fn extend(&mut self, other: &Stroke, last: &Stroke) { 
        self.to = last.to;
        self.count += other.count + last.count - 2;
        if last.up {
            self.high = last.high;
        } else { 
            self.low = last.low;
        }
    }

    pub fn check_include(&self, other: Stroke) -> Include {
        if self.up { 
            if other.low > self.low { 
                Include::Left
            } else if other.low < self.low { 
                Include::Right
            } else { 
                Include::Equal
            }
        } else if other.high < self.high { 
            Include::Left
        } else if other.high > self.high { 
            Include::Right
        } else { 
            Include::Equal
        }
    }
}

/// 分笔器
pub trait Judger {
    fn step(&mut self, stroke: Stroke, gaps: &IndexMap<usize, (f32, f32)>);
    fn get_strokes(&mut self) -> &Vec<Stroke>;
    fn get_vertexes(&mut self) -> Vec<Vertex> {
        let strokes = self.get_strokes();
        let mut vertices = Vec::with_capacity(strokes.len());
        
        for (i, stroke) in strokes.iter().enumerate() {
            // 只添加起点（除了第一笔）
            if i == 0 {
                let start_edge = if stroke.up { Edge::Trough } else { Edge::Peak };
                vertices.push(Vertex {
                    index: stroke.from,
                    value: if stroke.up { stroke.low } else { stroke.high },
                    edge: start_edge,
                    count: stroke.count,
                });
            }
            
            // 总是添加终点
            let end_edge = if stroke.up { Edge::Peak } else { Edge::Trough };
            vertices.push(Vertex {
                index: stroke.to,
                value: if stroke.up { stroke.high } else { stroke.low },
                edge: end_edge,
                count: stroke.count,
            });
        }
        
        vertices
    }
}

pub enum State {
    Empty,
    Inited,
}

pub struct DefaultJudger {
    strokes: Vec<Stroke>,
    leap: bool,
    state: State,
}
impl Judger for DefaultJudger {
    fn step(&mut self, mut stroke: Stroke, gaps: &IndexMap<usize, (f32, f32)>) {
        match self.state { 
            State::Empty => { 
                self.strokes.push(stroke);
                self.state = State::Inited;
            },
            State::Inited => { 
                if let Some(last_stroke) = self.strokes.last() { 
                    match last_stroke.check_include(stroke) { 
                        Include::Left => { 
                            self.strokes.push(stroke);
                        },
                        Include::Right | Include::Equal => {
                            if let Some(gap) = gaps.last() {
                                if self.leap && *gap.0 >= stroke.from && stroke.count < BI_COUNT {
                                    stroke.mark_done();
                                } 
                            };
                            let need_extend = match (last_stroke.is_done(), self.strokes.len() > 1) {
                                // 检查前一个是否未完成，只要有一个未完成就要合并笔进行延申
                                (true, true) => {
                                    let prev_stroke = &self.strokes[self.strokes.len() - 2];
                                    let extend_prev = !prev_stroke.is_done();
                                    
                                    if !extend_prev { // 前两笔都是完成笔，则追加新笔
                                        self.strokes.push(stroke);
                                    }
                                    extend_prev
                                },
                                
                                // 只有一个完成笔，直接追加新笔
                                (true, false) => {
                                    self.strokes.push(stroke);
                                    false
                                }
                                
                                // 右包含未完成笔，需要合并笔进行延申
                                (false, true) => true,
                                
                                // 右包含仅有的一个未完成笔，直接移除
                                (false, false) => {
                                    self.strokes.pop();
                                    self.strokes.push(stroke);
                                    false
                                }
                            };

                            if need_extend {
                                if let Some(ext) = self.try_extend(stroke) {
                                    self.step(ext, gaps);
                                }
                            }
                        },
                    }
                }
            },
        }

    }
    
    fn get_strokes(&mut self) -> &Vec<Stroke> { 
        while let Some(stroke) = self.strokes.last() {
            if !stroke.is_done() {
                self.strokes.pop();
            } else {
                break;
            }
        }
        &self.strokes 
    }
}

impl DefaultJudger {
    pub fn new(leap: bool, capacity: usize) -> Self { DefaultJudger { leap, strokes: Vec::with_capacity(capacity), state: State::Empty } }
    
    // 新笔反包最后一笔，根据最近3-4笔的情况进行合并
    pub fn try_extend(&mut self, stroke: Stroke) -> Option<Stroke> {
        let len = self.strokes.len();
        if let Some(last_stroke) = self.strokes.pop() {
            if let Some(prev_stroke) = self.strokes.last_mut() {
                match prev_stroke.check_include(last_stroke) {
                    Include::Left => {
                        // 如果最后一笔被前笔左包含，则将此两笔和新笔合并成一笔返回
                        prev_stroke.extend(&last_stroke, &stroke);
                        if self.strokes.len() == 1 {
                            self.state = State::Empty;
                        }
                        return self.strokes.pop()
                    },
                    Include::Right | Include::Equal => {
                        // 如果最后一笔右包含前笔，则将前三笔合并，返回新笔
                        // 如果这里只有两笔则丢弃第一笔，第二笔如果未成笔，也丢弃
                        if len == 2 {
                            if !prev_stroke.is_done() {
                                self.strokes.pop();
                            } 
                            self.strokes.push(stroke);
                            return None;
                        } else {
                            let prev_stroke = self.strokes.pop().unwrap();
                            if let Some(former_stroke) = self.strokes.last_mut() {
                                former_stroke.extend(&prev_stroke, &last_stroke);
                                return Some(stroke);
                            }
                        }
                    },
                }
            }
        }
        None
    }

}
