#![allow(dead_code)]
use std::collections::HashMap;
use std::os::raw::{c_int, c_float};
use std::cmp::Ordering;

use std::fs::File;
use std::io::prelude::*;

pub fn log(msg: &str) {
    let mut file = File::create("c:/tdx/demo.txt").unwrap();
    file.write_all(msg.as_bytes()).unwrap();
}

pub enum Deal {
    Buy1, Sell1,
    Buy2, Sell2,
    Buy3, Sell3,
    Buy23, Sell23
}

impl Deal {
    pub fn value(&self) -> f32 {
        match *self {
            Self::Buy1 => 1.0,
            Self::Sell1 => -1.0,
            Self::Buy3 => 3.0,
            Self::Sell3 => -3.0,
            Self::Buy2 => 2.0,
            Self::Sell2 => -2.0,
            Self::Buy23 => 23.0,
            Self::Sell23 => -23.0
        }
    }
}

// 行情
#[derive(Debug)]
pub struct BarChart {
    pub bars: Vec<Bar>
}

impl BarChart {
    pub fn with_capacity(capacity: c_int) -> Self {
        Self { bars: Vec::with_capacity(capacity as usize) }
    }

    pub fn add(&mut self, high: c_float, low: c_float) {
        let new_bar = if let Some(bar) = self.bars.last_mut() {
            match (high.total_cmp(&bar.high), low.total_cmp(&bar.low)) {
                (Ordering::Greater, Ordering::Greater) => {
                    Bar {high, low, trend:Trend::Advance, merged: false, merge_index: -1}
                },
                (Ordering::Less, Ordering::Less) => {
                    Bar {high, low, trend:Trend::Decline, merged:false, merge_index: -1}
                },
                _ => {
                    let (high, low) = match bar.trend {
                        Trend::Advance => (high.max(bar.high), low.max(bar.low)),
                        Trend::Decline => (high.min(bar.high), low.min(bar.low))
                    };

                    if !bar.merged {
                        bar.merged = true;
                        bar.merge_index = 0;
                    }
                    Bar {high, low, merge_index: bar.merge_index + 1, ..*bar}
                }
            }
        } else  {
            Bar {high, low, trend:Trend::Advance, merged:false, merge_index: -1}
        };
        if new_bar.merged {  // 向前重置合并状态、索引和高低点
            for bar in self.bars.iter_mut().rev() {  
                bar.high = new_bar.high;  
                bar.low = new_bar.low;  
                if bar.counted() {  
                    break;  
                }  
            }  
        }
        self.bars.push(new_bar);

    }

    pub fn find_pole(&self, std: bool) -> Vec<Pole> {
        let mut poles: Vec<Pole> = Vec::with_capacity(self.bars.capacity() / 5);

        let mut count = 0;
        let mut trend = (&self.bars[0]).trend;
        for (index, bar) in self.bars.iter().enumerate() {
            if bar.trend == trend {
                if bar.counted() {
                    count += 1;
                }
            } else {
                poles.push((&self.bars[index - 1]).as_end_pole(index - 1, count));
                count = 2;
                trend = bar.trend;
            }
        }
        
        if let (Some(bar), Some(pole)) = (self.bars.last(), poles.last()) {
            if pole.frac != bar.trend.end_frac() {
                poles.push(bar.as_end_pole(self.bars.len() - 1, count));
            }
        }
        if std {
            std_poles(&poles)
        } else {
            poles
        }
    }

}

fn std_poles(poles: &Vec<Pole>) -> Vec<Pole> {
    let mut new_poles: Vec<Pole> = Vec::with_capacity(poles.capacity());
    for pole in poles {
        let mut count = pole.count;
        loop {
            let mut removed_count = 0;
            if new_poles.len() > 2 {
                let poles_len = new_poles.len();
                match (new_poles.get(poles_len -3), new_poles.get(poles_len - 2), new_poles.get(poles_len -1)) {
                    (Some(p1), Some(p2), Some(p3)) if (p3.count < 5 || p2.count < 5) && ((pole.frac == 1 && pole.value >= p2.value) || (pole.frac != 1 && pole.value <= p2.value)) => {
                        if (pole.frac == 1 && p3.value >= p1.value) || (pole.frac != 1 && p3.value <= p1.value) {
                            // 上升时低点抬高或下降时高点下降
                            removed_count = p2.count + p3.count - 2;
                            new_poles.drain(poles_len-2..);
                        } else {
                            // 上升时低点下降或下降时高点抬高
                            removed_count = (p1.count + p2.count - 2) * -1;
                            new_poles.drain(poles_len - 3..poles_len - 1);
                        }
                    },
                    _ => {}
                };
            }

            if removed_count > 0 {
                count += removed_count
            } else if removed_count < 0 {
                if let Some(last) = new_poles.last_mut() {
                    last.count += removed_count * -1;
                }
            } else {
                break;
            }
        }
        new_poles.push(Pole { count, ..*pole });
    }
    return new_poles
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Pole {
    pub index: usize,
    pub frac: i8,
    pub count: c_int,
    pub value: c_float
}

impl Pole {
    pub fn trend(&self) -> Trend {
        if self.frac == 1 { Trend::Decline } else { Trend::Advance }
    }

    pub fn advance(&self) -> bool {
        self.frac != 1
    }
}

// 趋势
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trend {
    // 涨
    Advance,
    // 跌
    Decline
}

impl Trend {
    pub fn advance(&self) -> bool {
        if *self == Self::Advance { true } else { false }
    }

    fn start_frac(&self) -> i8 {
        if *self == Self::Advance { -1 } else { 1 }
    }

    fn end_frac(&self) -> i8 {
        if *self == Self::Advance { 1 } else { -1 }
    }

    fn merge_fn(&self) -> fn(f32, f32) -> f32 {
        if *self == Self::Advance { f32::max } else { f32::min }
    }
}

// K线
#[derive(Debug)]
pub struct Bar {
    pub high: c_float,
    pub low: c_float,
    pub trend: Trend,
    pub merged: bool,
    pub merge_index: i32,
}
impl Bar {
    fn start_pole_value(&self) -> f32 {
        if self.trend == Trend::Advance { self.low } else { self.high }
    }

    fn end_pole_value(&self) -> f32 {
        if self.trend == Trend::Advance { self.high } else { self.low }
    }

    fn as_start_pole(&self, index: usize, count: i32) -> Pole {
        let frac = self.trend.start_frac();
        let value = self.start_pole_value();
        Pole {index, frac, count, value}
    }

    fn as_end_pole(&self, index: usize, count: i32) -> Pole {
        let frac = self.trend.end_frac();
        let value = self.end_pole_value();
        Pole {index, frac, count, value}
    }

    fn counted(&self) -> bool {
        !self.merged || self.merge_index == 0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stroke {
    pub high: c_float,
    pub low: c_float,
    pub from: usize,
    pub to: usize,
    pub trend: Trend
}

impl Stroke {
    fn start_pole_value(&self) -> f32 {
        if self.trend.advance() { self.low } else { self.high }
    }

    fn end_pole_value(&self) -> f32 {
        if self.trend.advance() { self.high } else { self.low }
    }

    fn as_start_pole(&self) -> Pole {
        Pole { index: self.from, frac: self.trend.start_frac(), count:1, value: self.start_pole_value()}
    }

    fn high_index(&self) -> usize {
        if self.trend.advance() { self.to } else { self.from }
    }

    fn low_index(&self) -> usize {
        if self.trend.advance() { self.from } else { self.to }
    }

    fn highest_index(&self, second: &Self) -> usize {
        let highest = if self.high > second.high { self } else { second };
        highest.high_index()
    }

    fn lowest_index(&self, second: &Self) -> usize {
        let lowest = if self.low < second.low { self } else { second };
        lowest.low_index()
    }

    fn as_end_pole(&self) -> Pole {
        Pole { index: self.to, frac: self.trend.end_frac(), count:1, value: self.end_pole_value()}
    }

    fn top_fenxing(first: &Stroke, second: &Stroke, last: &Stroke) -> bool {
        last.low < second.low && last.high < second.high
            && second.high > first.high && second.low > first.low
    }

    fn bottom_fenxing(first: &Stroke, second: &Stroke, last: &Stroke) -> bool {
        last.low > second.low && last.high > second.high 
            && second.low < first.low && second.high < first.high
    }

    fn finish_segment(&self, first: &Stroke, second: &Stroke, last: &Stroke) -> bool {
        if self.trend == Trend::Advance {
            (*second != *first) && last.low < first.low || last.low < self.low
        } else {
            (*second != *first) && last.high > first.high || last.high > self.high
        }
    }

    fn finish_reverse_segment(&self, first: &Stroke, second: &Stroke, last: &Stroke) -> bool {
        second.low.min(last.low) < self.low.max(first.low) || second.high.max(last.high) > self.high.min(first.high)
    }
}

#[derive(Debug)]
pub struct Segment {
    pub start: Stroke,
    pub end: Stroke,
    pub turn: bool,
    pub first_feature: Option<Stroke>,
    pub last_feature: Option<Stroke>,
    pub last_origin_feature: Option<Stroke>,
    pub judge: Vec<Stroke>
}

enum Status {
    On,
    Gap,
    Finish
}

impl Segment {
    fn start_pole(&self) -> Pole {
        Pole{index: self.start.from, 
            frac: self.start.trend.start_frac(), count:1, 
            value: self.start.start_pole_value() }  
    }

    /// 反向线段被突破，原线段继续
    fn forward(&mut self, stroke: &Stroke) -> bool {
        if self.judge.len() < 3 && (self.advance() && stroke.high > self.end.high)
            || (!self.advance() && stroke.high > self.start.high) {
            self.end = stroke.clone();
            self.first_feature = Some(*self.judge.get(1).unwrap());
            self.clear_last_feature();
            self.turn = false;
            self.judge.clear();
            return true
        }
        false
    }

    /// 创新高新低，原先段继续，否则开始第一笔转折
    fn maintain_check(&mut self, stroke: &Stroke) {
        if (self.advance()&& stroke.high > self.end.high) // 向上创新高
            || (!self.advance() && stroke.low < self.end.low) { // 向下创新低
            self.end = *stroke;
            self.turn = false;
            if let Some(last) = self.judge.last() {
                self.first_feature = Some(*last);
                self.set_last_feature(*last);
            }
            self.judge.clear();
        } else {
            self.turn = true;
            self.judge.push(*stroke);
        }
    }

    fn end_pole(&self) -> Pole {
        Pole{index: self.end.to, 
            frac: self.start.trend.end_frac(), count:1, 
            value: self.start.end_pole_value() }
    }

    fn check_status(&mut self) -> Status {
        let mut include_index = 0;
        let mut status = Status::On;
        if let (Some(first), Some(second), Some(lo_feature)) = (self.first_feature, self.last_feature.as_mut(), self.last_origin_feature) {
            for (i, new_feature) in self.judge.iter().enumerate().filter(|p| p.0 % 2 == 1 ) {
                if second.high.total_cmp(&new_feature.high) != second.low.total_cmp(&new_feature.low) { // 处理包含
                    second.low = self.start.trend.merge_fn()(second.low, new_feature.low);
                    second.high = self.start.trend.merge_fn()(second.high, new_feature.high);
                    second.to = new_feature.to;
                    include_index = i + 1;
                } else {
                    let advance = self.start.trend.advance();                
                    if self.start.finish_segment(&first, second, new_feature) {
                        status = Status::Finish;
                    } else if (advance && Stroke::top_fenxing(&first, second, new_feature))
                        || (!advance && Stroke::bottom_fenxing(&first, second, new_feature)) {
                        if (advance && lo_feature.low > first.high) 
                            || (!advance && lo_feature.high < first.low) { // 分型一、二元素有缺口
                            status = Status::Gap;
                        } else {
                            status = Status::Finish;
                        }
                    }
                    break;                      
                }
            }
        }
        if include_index > 0 {
            self.judge.drain(..include_index);
        }
        status
    }

    pub fn new(stroke:Stroke) -> Self {
        Self {
            start: stroke.clone(), end: stroke.clone(), turn: false, first_feature: None, last_feature: None, last_origin_feature: None,
            judge: Vec::with_capacity(5)
        }
    }
    pub fn is_reverse(&self, stroke: &Stroke) -> bool {
        stroke.trend != self.start.trend
    }

    fn set_last_feature(&mut self, stroke: Stroke) {
        self.last_feature = Some(stroke);
        self.last_origin_feature = Some(stroke.clone());
    }

    fn swapin_feature(&mut self, stroke: Stroke) {
        if self.first_feature.is_none() {
            self.first_feature = Some(stroke);
            self.set_last_feature(stroke);
        } else if self.last_feature.is_none() {
            self.last_feature = Some(stroke);
            self.set_last_feature(stroke);
        } else {
            self.first_feature = self.last_feature;
            self.set_last_feature(stroke);
        }
    }

    fn check_reverse_status(&self, current: &Segment) -> Status {
        let mut features = Vec::new();
        let mut last: Option<Stroke> = None;
        for (_, v) in current.judge.iter().enumerate().filter(|p| p.0 % 2 == 0) {
            let mut next = true;
            if let Some(mut prev) = last {
                if prev.high.total_cmp(&v.high) != prev.low.total_cmp(&v.low) { // 处理包含
                    prev.high = self.start.trend.merge_fn()(prev.high, v.high);
                    prev.low = self.start.trend.merge_fn()(prev.low, v.low);
                    prev.to = v.to;
                    next = false;
                } else {
                    features.push(prev);
                }
            } 
            if next {
                last = Some(v.clone());
            }
        }

        if let (Some(first), Some(second), Some(latest)) = (current.first_feature, last, current.judge.last()) {
            if self.start.finish_reverse_segment(&first, &second, latest) {
                return Status::Finish
            }
            features.push(second);
        }

        let mut index = 0;
        let fl = features.len();
        while fl > 2 && (index + 2) <= fl {
            if let (Some(first), Some(second), Some(last)) = (features.get(index), features.get(index+1), features.get(index + 2)) {
                if (!self.start.trend.advance() && Stroke::bottom_fenxing(first, second, last))
                    || (self.start.trend.advance() && Stroke::top_fenxing(first, second, last)) {
                    return Status::Finish
                }
            }
            index += 1;
        }

        Status::On
    }

    fn advance(&self) -> bool {
        self.start.trend.advance()
    }

    fn check_features(&self, stroke: &Stroke, is_low: bool) -> bool {
        if let (Some(first), Some(last)) = (self.first_feature, self.last_feature) {
            let condition = if is_low {
                stroke.low < first.low && stroke.low < last.low && stroke.from != self.end.to
            } else {
                stroke.high > first.high && stroke.high > last.high && stroke.from != self.end.to
            };
            first != last && condition
        } else {
            false
        }
    }

    fn broken_by(&mut self, stroke: &Stroke) -> bool {
        let broken = if self.advance() {
            stroke.low < self.start.low && (!self.check_features(stroke, true))
        } else {
            stroke.high > self.start.high && (!self.check_features(stroke, false))
        };
        if broken {
            self.turn = true;
            if self.judge.is_empty() {
                self.set_last_feature(*stroke);
            } else {
                self.judge.push(*stroke);
            }
        } else {
            if self.turn { // 转折
                self.judge.push(*stroke);
            } else { // 未发生转折
                self.swapin_feature(*stroke);
            }
        }
        broken
    }

    fn clear_last_feature(&mut self) {
        self.last_feature = None;
        self.last_origin_feature = None;
    }

}

pub struct Segmenter {
    pub poles: Vec<Pole>,
    pub current: Option<Segment>,
    pub reverse: Option<Segment>
}

impl Segmenter {
    pub fn new() -> Self {
        Self { poles: Vec::with_capacity(10), current: None, reverse: None}
    }

    pub fn finish(&mut self) {
        let curr = self.current.as_ref().unwrap();
        let mut strokes = Vec::with_capacity(curr.judge.len());
        for s in &curr.judge {
            strokes.push(s.clone());
        }
        self.poles.push(curr.start.as_start_pole());
        let last = curr.last_feature;

        self.current = None;
        if let Some(s) = last {
            self.add_stroke(s);
        }
        for j in strokes {
            self.add_stroke(j);
        }
    }
    pub fn get_segments(&self) -> Vec<Pole> {
        let mut result = self.poles.clone();

        if let Some(curr) = &self.current {
            result.push(curr.start.as_start_pole());
            result.push(curr.end.as_end_pole())
        }
        if let Some(rseg) = &self.reverse { // 反向未确认线段，设置 count = 2 区分
            let mut pole = rseg.end.as_end_pole();
            pole.count = 2;
            result.push(pole);
        }
        result
    }

    pub fn add_stroke(&mut self, stroke: Stroke) {
        let mut check_status = false;
        if self.reverse.is_some() {
            if let Some(seg) = &mut self.current {
                if !seg.is_reverse(&stroke) {
                    if seg.forward(&stroke) { // 原方向突破，线段继续
                        self.reverse = None;
                        return
                    }
                }
                seg.judge.push(stroke);
                check_status = true;
            }
        } else {
            if let Some(seg) = &mut self.current {
                if seg.is_reverse(&stroke) { // 反向笔
                    if seg.broken_by(&stroke) {
                        self.finish();
                        return;
                    } else if seg.turn { // 转折
                        check_status = true;
                    }
                } else {
                    seg.maintain_check(&stroke);
                }
            } else {
                self.current = Some(Segment::new(stroke));
            }
        }
        if check_status {
            if let (Some(rseg), Some(curr)) = (&mut self.reverse, &self.current) {
                match rseg.check_reverse_status(curr) {
                    Status::Finish => self.finish_by_gap(),
                    Status::Gap => {},
                    Status::On => {}
                }
            } else if let Some(seg) = &mut self.current {
                match seg.check_status() {
                    Status::Finish => self.finish(),
                    Status::Gap => self.gap(),
                    Status::On => {}
                }
            };
        }
    }

    /// 设置反向段
    fn gap(&mut self) {
        if let Some(curr) = &self.current {
            if let Some(last) = curr.last_feature {
                let mut seg = Segment::new(last);
                seg.end = *curr.judge.last().unwrap();
                self.reverse = Some(seg);
            }
        }
    }

    /// 被反向段结束
    fn finish_by_gap(&mut self) {
        let mut new_segment = None;
        let mut new_judge = Vec::new();

        if let (Some(seg), Some(_rseg)) = (&mut self.current, &mut self.reverse ) {
            
            self.poles.push(seg.start_pole());
            let advance = seg.advance();
            if let (Some(last_origin), Some(last)) = (seg.last_origin_feature, seg.last_feature.as_mut()) {
                if last_origin.from != last.from || last_origin.to != last.to {
                    last.from = last_origin.from;
                    if advance {
                        last.high = last_origin.high;
                    } else {
                        last.low = last_origin.low;
                    }
                }
                new_segment = Some(Segment::new(*last));
            }
            for v in seg.judge.iter() {
                new_judge.push(*v);
            }
        }
        self.reverse = None;
        self.current = new_segment;
        for v in new_judge {
            self.add_stroke(v);
        }
    }

    pub fn get_pivots(segments: &Vec<Pole>, strokes: &Vec<Stroke>) -> (Vec<Pivot>, HashMap<usize, Deal>) {
        let mut pivots: Vec<Pivot> = Vec::with_capacity(10);
        let mut seg_index = 0;
        // let last_index = segments.len() - 2;
        let mut bp = HashMap::new();
        while seg_index < (segments.len()) {
            if let (Some(seg_start), seg_end) = (segments.get(seg_index), segments.get(seg_index + 1)) {
                let seg_strokes:Vec<&Stroke> = strokes.iter().filter(|s: &&Stroke| {
                    s.from >= seg_start.index && (seg_end.is_none() || s.to <= seg_end.unwrap().index) // 在段开始结束之间
                     && s.trend != seg_start.trend() // 找反向笔
                }).collect();

                let advance = seg_start.advance();
                if seg_strokes.is_empty() && pivots.len() > 0 {
                    let ps = &strokes[strokes.len() - 2];
                    let s = &strokes[strokes.len() - 1];
                    let p = &pivots[pivots.len() - 1];
                    if s.from == seg_start.index && p.leave(s) && ps.from == p.end {
                        bp.insert(s.to, if advance { Deal::Sell3 } else { Deal::Buy3 });
                    }
                    seg_index += 1;
                    continue;
                }
                
                let mut stroke_index = 0;
                let mut has_pivot = false;
                let slen = seg_strokes.len();
                let mut pivot_len = 0;
                while slen > 0 && stroke_index <= (slen - 1) {
                    let mut next_pivot = true;
                    if let (Some(first), next) = (seg_strokes.get(stroke_index), seg_strokes.get(stroke_index + 1)) {
                        if has_pivot {                        
                            if let Some(p) = pivots.last_mut() {
                                if p.leave(first) { // 离开中枢
                                    next_pivot = true;
                                    has_pivot = false;
                                    if (advance && first.low > p.high) || (!advance && first.high < p.low) {
                                        bp.insert(first.to, if advance { Deal::Buy3 } else { Deal::Sell3});
                                    } else if (advance && first.high < p.low) || (!advance && first.low > p.high) {
                                        bp.insert(first.from, if advance { Deal::Sell3 } else { Deal::Buy3});
                                    }
                                } else if seg_end.is_some() && first.to <= seg_end.unwrap().index { // 中枢震荡
                                    next_pivot = false;
                                    p.congestion(first);
                                }
                            }
                        } else if stroke_index == 0 && bp.contains_key(&seg_start.index) {
                            let p = &pivots[pivots.len() - 1];
                            if advance {
                                if p.low > first.high {
                                    bp.insert(first.from, Deal::Sell3);
                                    bp.insert(first.to, Deal::Buy2);
                                } else {
                                    bp.insert(first.to, if first.low > p.high { Deal::Buy23 } else { Deal::Buy2 });
                                }
                            } else {
                                if p.high < first.low {
                                    bp.insert(first.from, Deal::Buy3);
                                    bp.insert(first.to, Deal::Sell2);
                                } else {
                                    bp.insert(first.to, if first.high < p.low { Deal::Sell23 } else { Deal::Sell2 });
                                }
                            }
                        }
                        if next_pivot && pivot_len > 1 {
                            let (p, pp) = (&pivots[pivot_len - 2], &pivots[pivot_len - 1]);
                            if (advance && p.high < first.low && pp.high < p.low) || (!advance && p.low > first.high && pp.low > p.high) {
                                bp.insert(first.from, if advance { Deal::Sell1 } else { Deal::Buy1 });
                                if let Some(n) = next {
                                    if (advance && n.high < first.high) || (!advance && n.low > first.low ) {
                                        bp.insert(n.from, if advance { Deal::Sell2 } else { Deal::Buy2 });
                                    }
                                }
                            }
                        }
                        if next_pivot && seg_end.is_some() && first.to <= seg_end.unwrap().index {
                            if let Some(second) = next {
                                if (advance && first.high >= second.low) || (!advance && first.low <= second.high) { // 中枢新生
                                    pivots.push(Pivot::from(first, second));
                                    pivot_len += 1;
                                    has_pivot = true;
                                    stroke_index += 1;
                                }
                            }
                        }
                        stroke_index += 1;
                        
                    }
                    // 中枢扩展判定和处理
                    if pivot_len > 1 {
                        check_pivot_extension(&mut pivots, &mut pivot_len);
                    }
                }
                if pivot_len > 1 && seg_end.is_some() {
                    bp.insert(seg_end.unwrap().index, if advance { Deal::Sell1 } else { Deal::Buy1 });
                }
            }
            seg_index+= 1;
        }
        (pivots, bp)
    }
}

fn check_pivot_extension(pivots: &mut Vec<Pivot>, pivot_len: &mut usize) {
    let merged_pivot = if let (Some(prev), Some(next)) = (pivots.get(pivots.len() - 2), pivots.last()) {
        if next.highest > prev.lowest && next.lowest < prev.low { // 后中枢低于前中枢 
            Some(Pivot {
                high: next.highest.max(prev.lowest), low: prev.lowest.min(next.highest), trend: prev.trend,
                start: prev.lowest_index, end: next.highest_index,
                lowest: next.lowest, lowest_index: next.lowest_index, 
                highest: prev.highest.max(next.highest), highest_index: if next.highest > prev.highest { next.highest_index } else { prev.highest_index } })
        } else if next.lowest < prev.highest && next.highest > prev.high { // 后中枢高于前中枢
            Some(Pivot {
                high: next.lowest.max(prev.highest), low: prev.highest.min(next.lowest), trend: prev.trend,
                start: prev.highest_index, end: next.lowest_index,
                highest: next.highest, highest_index: next.highest_index, 
                lowest: prev.lowest.min(next.lowest), lowest_index: if prev.lowest < next.lowest { prev.lowest_index } else { next.lowest_index } })
        } else {
            None
        }
    } else {
        None
    };
    if let Some(merged) = merged_pivot {
        pivots.pop();
        pivots.pop();
        pivots.push(merged);
        *pivot_len -= 1;
    }
}

#[derive(Debug)]
pub struct Pivot {
    pub highest: c_float,
    pub highest_index: usize,
    pub lowest_index: usize,
    pub lowest: c_float,
    pub high: c_float,
    pub low: c_float,
    pub start: usize,
    pub end: usize,
    pub trend: Trend
}

impl Pivot {
    fn from(first: &Stroke, second: &Stroke) -> Pivot {
        Pivot{high: first.high.min(second.high)
            , highest: first.high.max(second.high)
            , low: first.low.max(second.low)
            , lowest: first.low.min(second.low)
            , start:first.from
            , end: second.to
            , trend: first.trend
            , highest_index: first.highest_index(second)
            , lowest_index: first.lowest_index(second)
        }
    }

    /// 中枢离开
    fn leave(&self, first: &Stroke) -> bool {
        first.low > self.high || first.high < self.low 
    }

    /// 中枢震荡，更新结束位置，中枢最高低值及位置
    fn congestion(&mut self, first: &Stroke) {
        self.end = first.to;
        if first.high > self.highest {
            self.highest = first.high;
            self.highest_index = first.high_index();
        }
        if first.low < self.lowest {
            self.lowest = first.low;
            self.lowest_index = first.low_index();
        }
    }
}