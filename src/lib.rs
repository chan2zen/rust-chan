#![allow(non_snake_case)]
mod market;

use std::os::raw::{c_int, c_float, c_ushort};

use market::{BiMode, Market, PivotFinder, PivotMode};

#[cfg(test)]
mod tests;

use std::fs::OpenOptions;
use std::io::prelude::*;

#[allow(unused_macros)]
macro_rules! log {
    ($format:tt, $($arg:expr),*) => {
        write_log_entry(format_args!($format, $($arg),*))
    };
}
pub fn write_log_entry(entry: std::fmt::Arguments) {
    let mut file = OpenOptions::new().append(true).create(true)
        .open("c:/tdx/demo.txt").expect("failed to open log file");
    file.write_fmt(entry).expect("failed to write to log");
}

#[repr(C, packed(1))]
pub struct PluginTCalcFuncInfo {
    nFuncMark: c_ushort,
    pCallFunc: Option<PlugInFunc>,
}

type PlugInFunc = unsafe extern "C" fn(c_int, *mut c_float, *mut c_float, *mut c_float, *mut c_float);

/**
 * 严格笔，推笔（不允许顶底包含），缺口直接突破成笔
 * 使用千位数值拆分，
 * bi_mode 使用千位拆分，通过位运算，默认严格笔 1，+次高成笔 2， +缺口突破 4，值相加组合
 * pivot mode 使用十位， x0x 表示笔, x1x 表示段, x2x 表示趋势
 * 日志使用个位，xx0 表示无日志(除2的余数为0)，xx1 表示打印日志(除2的余数为1), 大于 2 表示不设进入段
 * 
 * pole 使用百位，1xx 表示极值, 2xx 表示极点类型，笔、段端点标识(-1,1;-100,100, -200(临时段端点))
 * 或者
 * mode=5, 小顶底，-1/1, 强顶底,-2/2
 * mode=4, 买卖点，4xx 表示买卖点
 * mode=2，中枢高, 2xx 表示中枢高
 * mode=3，中枢低, 如果中枢扩展，为负值; 3xx 表示中枢低
 * mode=1（默认值), 中枢位置(-2,2);  1xx 表示中枢起始位置
 */
struct ZigzagConfig {
    bi_mode: BiMode,
    log: bool,
    segment_entry: bool,
    pivot_mode: PivotMode,
    pole_value_mode: bool,
    pole_edge_mode: bool,

    signal: bool,
    zg: bool,
    zd: bool,
    fx: bool, // 小顶底
    pivot_position: bool,
}

impl ZigzagConfig {
    fn new(mode: i32, pivot: bool) -> Self {
        let bi_mode = BiMode::new(mode);

        let mut zd = false;
        let mut zg = false;
        let mut fx = false;
        let mut signal = false;
        let mut pivot_position = false;
        let mut pole_value_mode = false;
        let mut pole_edge_mode = false;
        let mode_value = mode % 1000 / 100;
        if pivot {
            signal = mode_value == 4;
            zd = mode_value == 3;
            zg = mode_value == 2;
            pivot_position = mode_value == 1;
            fx = mode_value == 5;
        } else {
            pole_value_mode = mode_value == 1;
            pole_edge_mode = mode_value == 2;
        }
        
        let pivot_mode = PivotMode::new(mode);
        let log = mode % 10 % 2 == 1;
        let segment_entry = mode % 10 / 2 == 0;
        ZigzagConfig {
            bi_mode,
            log,
            segment_entry,
            pivot_mode,
            pole_value_mode,
            pole_edge_mode,
            signal,
            zg,
            zd,
            pivot_position,
            fx,
        }
    }
}

pub unsafe extern "C" fn zigzag(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let config = ZigzagConfig::new(*mode as i32, false);
    let DataLen = DataLen;
    let market = Market::with_bi_mode(DataLen as usize, pfINa_high, pfINb_low, config.bi_mode);
    if config.log {
        log!("\ncreate_market!({},{:?},{:?})\n", DataLen, market.high, market.low);
    }
    let zr = market.zigzag(config.pivot_mode, config.log, config.segment_entry);
    if config.log {
        log!("\nzr: {:?}\n", zr);
    }
    let zigzag = match config.pivot_mode {
        PivotMode::BI => zr.bi_zigzag.unwrap(),
        PivotMode::DUAN => zr.duan_zigzag.unwrap(),
        PivotMode::TREND => zr.trend_zigzag.unwrap(),
    };
    let poles = &zr.spins.unwrap();
    for pole in poles {
        let value = if config.pole_value_mode {
            pole.value
        } else {
            pole.edge as isize as c_float * 
            if (config.pivot_mode.is_bi() && pole.segmented) 
                || (config.pivot_mode.is_duan() && pole.trended)
                || (config.pivot_mode.is_trend() && pole.trend_upgraded) { 
                100. 
            } else if (config.pivot_mode.is_bi() && !pole.segmented && !pole.trended && !pole.trend_upgraded)
                || (config.pivot_mode.is_duan() && pole.segmented)
                || (config.pivot_mode.is_trend() && pole.trended) { 1. } else { 0. }
            * if zigzag.tip > 0 && pole.index == zigzag.tip { 200. } else { 1. }
        };

        *pfOUT.offset(pole.index as isize) = value;
    }
    if config.pivot_mode == PivotMode::BI && config.pole_edge_mode { // 将最后分段的 state 写入最后一个极点前
        for pole in &zigzag.intermediate {
            *pfOUT.offset(pole.index as isize) = pole.edge as isize as c_float * 3.;
        }
        if let Some(pole) = poles.last() {
            if pole.index > 0 { *pfOUT.offset(pole.index as isize - 1) = zigzag.state as isize as c_float; }
        }
    } else {
        for pole in &zigzag.intermediate {
            *pfOUT.offset(pole.index as isize) = pole.value;
        }
    }
}

pub unsafe extern "C" fn pivot(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let config = ZigzagConfig::new(*mode as i32, true);
    let market = Market::with_bi_mode(DataLen as usize, pfINa_high, pfINb_low, config.bi_mode);
    if config.fx {
        for (k, v) in market.fx_indexes.iter() {
            *pfOUT.offset(*k as isize) = *v as c_float;
        }
        return;
    }
    let zr= market.zigzag(config.pivot_mode, config.signal, config.segment_entry);

    let zigzag = match config.pivot_mode {
        PivotMode::BI => zr.bi_zigzag.unwrap(),
        PivotMode::DUAN => zr.duan_zigzag.unwrap(),
        PivotMode::TREND => zr.trend_zigzag.unwrap(),
    };
    if config.signal {
        if let Some(signals) = zigzag.signals {
            for (index, signal) in signals.iter() {
                *pfOUT.offset(*index as isize) = *signal as i32 as c_float;
            }
        }
        return;
    }
    // log!("\nzigzag: {:?}\n", zigzag);
    
    for pivot in zigzag.pivots {
        if config.zg {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.high() as c_float * if pivot.extended { -1. } else { 1. };
            }
        } else if config.zd {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.low() as c_float * if pivot.extended { -1. } else { 1. };
            }
        } else if config.pivot_position {
            *pfOUT.offset(pivot.start() as isize) = -2.;
            *pfOUT.offset(pivot.end() as isize) = 2.;
        } 
    }
}

pub unsafe extern "C" fn pivot_new(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let config = ZigzagConfig::new(*mode as i32, true);
    let market = Market::with_bi_mode(DataLen as usize, pfINa_high, pfINb_low, config.bi_mode);
    let mut poles = market.tracer.poles();
    if config.pivot_mode == PivotMode::DUAN {
        market.stain_duan(&mut poles);
    }
    let finder = PivotFinder::new();
    let entries = finder.find(&poles);
    for entry in entries {
        if config.signal {
            if let Some(signals) = entry.signals {
                for (index, signal) in signals.iter() {
                    *pfOUT.offset(*index as isize) = *signal as i32 as c_float;
                }
            }
            continue;
        }        
        if let Some(pivot) = entry.pivot {
            if config.zg {
                for i in pivot.start()..=pivot.end() {
                    *pfOUT.offset(i as isize) = pivot.high() as c_float * if pivot.extended { -1. } else { 1. };
                }
            } else if config.zd {
                for i in pivot.start()..=pivot.end() {
                    *pfOUT.offset(i as isize) = pivot.low() as c_float * if pivot.extended { -1. } else { 1. };
                }
            } else if config.pivot_position {
                *pfOUT.offset(pivot.start() as isize) = -2.;
                *pfOUT.offset(pivot.end() as isize) = 2.;
            } 
        }
    }
}
    
static mut G_CALC_FUNC_SETS: [PluginTCalcFuncInfo; 4] = [
    PluginTCalcFuncInfo {
        nFuncMark: 3,
        pCallFunc: Some(pivot_new),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 2,
        pCallFunc: Some(pivot),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 1,
        pCallFunc: Some(zigzag),
    },
    PluginTCalcFuncInfo {
        nFuncMark: 0,
        pCallFunc: None,
    },
];

#[no_mangle]
#[allow(static_mut_refs)]
pub unsafe extern "C" fn RegisterTdxFunc(pFun: *mut *mut PluginTCalcFuncInfo) -> c_int {
    if (*pFun).is_null() {
        *pFun = G_CALC_FUNC_SETS.as_mut_ptr();
        1
    } else {
        0
    }
}