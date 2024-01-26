#![allow(non_snake_case)]

mod chan;
mod ttd;

use std::os::raw::{c_int, c_float, c_ushort};
use chan::{BarChart, Pole, Stroke, Trend, Segmenter, log, Deal};
use std::collections::HashMap;

#[repr(C, packed(1))]
pub struct PluginTCalcFuncInfo {
    nFuncMark: c_ushort,
    pCallFunc: Option<PlugInFunc>,
}

type PlugInFunc = unsafe extern "C" fn(c_int, *mut c_float, *mut c_float, *mut c_float, *mut c_float);

#[no_mangle]
pub unsafe extern "C" fn TestPlugin1(DataLen: c_int, pfOUT: *mut c_float, _pfINa: *mut c_float, _pfINb: *mut c_float, _pfINc: *mut c_float) {
    for i in 0..DataLen {
        *pfOUT.offset(i as isize) = i as c_float;
    }
}

#[no_mangle]
pub unsafe extern "C" fn TestPlugin2(DataLen: c_int, pfOUT: *mut c_float, pfINa: *mut c_float, pfINb: *mut c_float, pfINc: *mut c_float) {
    for i in 0..DataLen {
        *pfOUT.offset(i as isize) = (*pfINa.offset(i as isize) + *pfINb.offset(i as isize) + *pfINc.offset(i as isize)) / 3.0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Merged(count: c_int, out: *mut c_float, high: *mut c_float, low: *mut c_float, want: *mut c_float) {
    let mut chart = BarChart::with_capacity(count);
    for i in 0..count as isize {
        chart.add(*high.offset(i), *low.offset(i));
    }

    for i in 0..count as usize {
        let bar = chart.bars.get(i).unwrap();
        *out.offset(i as isize) = if bar.merged {
            match *want as i8 {
                1 => bar.high,
                -1 => bar.low,
                2 => match bar.merge_index {
                    0 => 1.0,
                    -1 => 0.0, 
                    _ => {
                        if i >=1 && *out.offset(i as isize - 1) == 2.0 {
                            *out.offset(i as isize - 1) = 0.0;
                        }
                        2.0
                    }
                },
                0 => match bar.trend {
                    chan::Trend::Advance => 1.0,
                    chan::Trend::Decline => -1.0
                }
                _ => 0.0
            }
        } else {
            0.0
        };
    }
}

#[no_mangle]
pub unsafe extern "C" fn FindBiPoles(count: c_int, out: *mut c_float, high: *mut c_float, low: *mut c_float, mode: *mut c_float) {
    let mut chart = BarChart::with_capacity(count);
    for i in 0..count as isize {
        chart.add(*high.offset(i), *low.offset(i));
    }

    let poles = chart.find_pole(*mode == 1.0);
    let last_index = if count > 5 { count as usize - 5 } else { count as usize - 1};
    for pole in poles {
        if pole.index >= last_index && pole.count < 5 {
            continue;
        }
        let frac_index = pole.index as isize;
        *out.offset(frac_index as isize) = pole.frac as c_float;
    }

}

#[no_mangle]
pub unsafe extern "C" fn FindSegPoles(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let strokes = build_strokes_from_frac(count, frac, low, high);
        
    let mut handler = Segmenter::new();
    for s in &strokes {
        handler.add_stroke(*s);
    }
    
    let seg_poles = handler.get_segments();
    
    for pole in &seg_poles {
        *out.offset(pole.index as isize) = (pole.frac * pole.count as i8) as c_float;
    }
    
}

fn build_strokes_from_frac(count: i32, frac: *mut f32, low: *mut f32, high: *mut f32) -> Vec<Stroke> {
    let mut poles: Vec<Pole> = Vec::with_capacity(count as usize / 4);
    for i in 0..count as isize {
        let curr_frac = unsafe { *frac.offset(i) } as i8;
        if curr_frac == 1 || curr_frac == -1 {
            let value = if curr_frac == -1 { unsafe { *low.offset(i) } } else { unsafe { *high.offset(i) }};
            poles.push(Pole {index: i as usize, frac: curr_frac as i8, count: 5, value: value});
        }
    }
    
    let mut strokes = Vec::with_capacity(poles.len());
    let mut prev = None;
    for (i, v) in poles.iter().enumerate() {
        if i == 0 {
            prev = Some(v);
        } else {
            if let Some(p) = prev {
                let stroke = if p.frac == -1 {
                    Stroke{high: v.value, low:p.value, from:p.index, to:v.index
                        , trend: Trend::Advance }
                } else {
                    Stroke{high: p.value, low:v.value, from:p.index, to:v.index
                        , trend: Trend::Decline }
                };
                strokes.push(stroke);
            }
            prev = Some(v);
        }
    }

    strokes
}

fn build_strokes_from_poles(count: i32, seg_poles: Vec<Pole>, low: *mut f32, high: *mut f32) -> Vec<Stroke> {
    let mut poles: Vec<Pole> = Vec::with_capacity(count as usize / 4);
    let map: HashMap<usize, i8> = seg_poles.into_iter().map(|pole| (pole.index, pole.frac)).collect();
    for i in 0..count as isize {
        let curr_frac = if let Some(frac) = map.get(&(i as usize)) {
            *frac
        } else {
            0
        };
        if curr_frac == 1 || curr_frac == -1 {
            let value = if curr_frac == -1 { unsafe { *low.offset(i) } } else { unsafe { *high.offset(i) }};
            poles.push(Pole {index: i as usize, frac: curr_frac as i8, count: 5, value: value});
        }
    }
    
    let mut strokes = Vec::with_capacity(poles.len());
    let mut prev = None;
    for (i, v) in poles.iter().enumerate() {
        if i == 0 {
            prev = Some(v);
        } else {
            if let Some(p) = prev {
                let stroke = if p.frac == -1 {
                    Stroke{high: v.value, low:p.value, from:p.index, to:v.index
                        , trend: Trend::Advance }
                } else {
                    Stroke{high: p.value, low:v.value, from:p.index, to:v.index
                        , trend: Trend::Decline }
                };
                strokes.push(stroke);
            }
            prev = Some(v);
        }
    }
    strokes
}

#[no_mangle]
pub unsafe extern "C" fn FindPivotBoundry(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let pivots = get_pivots(count, frac, low, high);
    
    for p in &pivots {
        *out.offset(p.start as isize) = 1.0;
        *out.offset(p.end as isize) = 2.0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FindPivotHigh(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let pivots = get_pivots(count, frac, low, high);
    
    for p in &pivots {
        for i in p.start..(p.end+1) {
            *out.offset(i as isize) = p.high;
        }
    }
}

fn get_pivots(count: i32, frac: *mut f32, low: *mut f32, high: *mut f32) -> Vec<chan::Pivot> {
    let strokes = build_strokes_from_frac(count, frac, low, high);
        
    let mut handler = Segmenter::new();
    for s in &strokes {
        handler.add_stroke(*s);
    }
    
    
    let seg_poles = handler.get_segments();
    
    let (pivots, _bp) = Segmenter::get_pivots(&seg_poles, &strokes);
    pivots
}

fn find_deal_points(count: i32, frac: *mut f32, low: *mut f32, high: *mut f32, bi: bool, debug: bool) -> HashMap<usize, Deal> {
    let mut strokes = build_strokes_from_frac(count, frac, low, high);
        
    let mut handler = Segmenter::new();
    strokes.iter().for_each(|s| handler.add_stroke(*s));
    
    let mut seg_poles = handler.get_segments();

    let msg = if debug {
        Some(format!("bi strokes: {:?}\r\nbi_poles: {:?}", &strokes, &seg_poles))
    } else {
        None
    };
    if !bi {
        strokes = build_strokes_from_poles(count, seg_poles, low, high);
        handler = Segmenter::new();
        strokes.iter().for_each(|s| handler.add_stroke(*s));
        seg_poles = handler.get_segments();
    }
    
    if debug {
        log(format!("{}\r\nseg strokes: {:?}\r\nseg_poles: {:?}", msg.unwrap(), &strokes, &seg_poles).as_str());
    }
    let (_pivots, bp) = Segmenter::get_pivots(&seg_poles, &strokes);
    bp
}

#[no_mangle]
pub unsafe extern "C" fn FindPivotLow(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let pivots = get_pivots(count, frac, low, high);
    
    for p in &pivots {
        for i in p.start..(p.end+1) {
            *out.offset(i as isize) = p.low;
        }
    }

}

#[no_mangle]
pub unsafe extern "C" fn FindStroke1Buy(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let deal_points = find_deal_points(count, frac, low, high, true, false);

    for (index, deal) in deal_points {
        *out.offset(index as isize) = deal.value()
    }
}

#[no_mangle]
pub unsafe extern "C" fn FindSeg1Buy(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let deal_points = find_deal_points(count, frac, low, high, false, false);

    for (index, deal) in deal_points {
        *out.offset(index as isize) = deal.value()
    }
}

#[no_mangle]
pub unsafe extern "C" fn LogStrokes(count: c_int, out: *mut c_float, frac: *mut c_float, high: *mut c_float, low: *mut c_float) {
    let deal_points = find_deal_points(count, frac, low, high, false, true);

    for (index, deal) in deal_points {
        *out.offset(index as isize) = deal.value()
    }
}

static mut G_CALC_FUNC_SETS: [PluginTCalcFuncInfo; 12] = [
    PluginTCalcFuncInfo {
        nFuncMark: 1,
        pCallFunc: Some(TestPlugin1),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 2,
        pCallFunc: Some(TestPlugin2),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 3,
        pCallFunc: Some(Merged),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 4,
        pCallFunc: Some(FindBiPoles),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 5,
        pCallFunc: Some(FindSegPoles),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 6,
        pCallFunc: Some(FindPivotBoundry)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 7,
        pCallFunc: Some(FindPivotHigh)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 8,
        pCallFunc: Some(FindPivotLow)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 9,
        pCallFunc: Some(FindStroke1Buy)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 10,
        pCallFunc: Some(FindSeg1Buy)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 11,
        pCallFunc: Some(LogStrokes)
    },
    PluginTCalcFuncInfo {
        nFuncMark: 0,
        pCallFunc: None,
    },
];


#[no_mangle]
pub unsafe extern "C" fn RegisterTdxFunc(pFun: *mut *mut PluginTCalcFuncInfo) -> c_int {
    if (*pFun).is_null() {
        *pFun = G_CALC_FUNC_SETS.as_mut_ptr();
        1
    } else {
        0
    }
}
