#![allow(non_snake_case)]
mod market;

use std::os::raw::{c_int, c_float, c_ushort};

use market::Market;

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
// 打印日志，获取输入输出，方便调试
const LOG_MODE: c_float = 9.;
// , mode=2，极值
const POLE_VALUE_MODE: c_float = 2.;
// mode=1（默认值), 笔、段端点标识(-1,1;-100,100, -200(临时段端点))
pub unsafe extern "C" fn zigzag(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let market = Market::new(DataLen as usize, pfINa_high, pfINb_low);
    if *mode == LOG_MODE {
        log!("\ncreate_market!({},{:?},{:?})\n", DataLen, market.high, market.low);
    }
    let zigzag = market.zigzag_with_flag(true);
    if *mode == LOG_MODE {
        log!("\nzigzag: {:?}\n", zigzag);
    }
    for pole in &zigzag.poles {
        let value = if *mode == POLE_VALUE_MODE {
            pole.value
        } else {
            pole.edge as isize as c_float * if pole.segmented { 100. } else { 1. }
            * if zigzag.tip > 0 && pole.index == zigzag.tip { 200. } else { 1. }
        };
        
        *pfOUT.offset(pole.index as isize) = value;
    }
    if *mode != POLE_VALUE_MODE { // 将最后分段的 state 写入最后一个极点前
        for pole in &zigzag.intermediate {
            *pfOUT.offset(pole.index as isize) = pole.edge as isize as c_float * 3.;
        }
        if let Some(pole) = &zigzag.poles.last() {
            if pole.index > 0 { *pfOUT.offset(pole.index as isize - 1) = zigzag.state as isize as c_float; }
        }
    } else {
        for pole in &zigzag.intermediate {
            *pfOUT.offset(pole.index as isize) = pole.value;
        }
    }
}
// mode=4, 买卖点
const SIGNAL_MODE: c_float = 4.;
// mode=2，中枢高,
const ZG_MODE: c_float = 2.;
// mode=3，中枢低, 如果中枢扩展，为负值;
const ZD_MODE: c_float = 3.;
// mode=1（默认值), 中枢位置(-2,2);  
pub unsafe extern "C" fn pivot(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let market = Market::new(DataLen as usize, pfINa_high, pfINb_low);
    let zigzag= market.zigzag_with_flag(*mode != SIGNAL_MODE);
    
    if let Some(signals) = zigzag.signals {
        for (index, signal) in signals.iter() {
            *pfOUT.offset(*index as isize) = *signal as i32 as c_float;
        }
        return;
    }
    for pivot in zigzag.pivots {
        if *mode == ZG_MODE {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.high() as c_float * if pivot.extended { -1. } else { 1. };
            }
        } else if *mode == ZD_MODE {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.low() as c_float * if pivot.extended { -1. } else { 1. };
            }
        } else {
            *pfOUT.offset(pivot.start() as isize) = -2.;
            *pfOUT.offset(pivot.end() as isize) = 2.;
        } 
    }
}
static mut G_CALC_FUNC_SETS: [PluginTCalcFuncInfo; 3] = [
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
pub unsafe extern "C" fn RegisterTdxFunc(pFun: *mut *mut PluginTCalcFuncInfo) -> c_int {
    if (*pFun).is_null() {
        *pFun = G_CALC_FUNC_SETS.as_mut_ptr();
        1
    } else {
        0
    }
}