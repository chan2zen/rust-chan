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
const LOG_MODE: c_float = 9.;
// mode=1（默认值), 笔、段端点标识(-1,1;-100,100), mode=2，极值
pub unsafe extern "C" fn zigzag(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let market = Market::new(DataLen as usize, pfINa_high, pfINb_low);
    let (zigzag, _signals, pivots) = market.zigzag();
    if *mode == LOG_MODE {
        log!("\ncreate_market!({},{:?},{:?})\nzigzag: {:?}\npivots: {:?}\n", DataLen, market.high, market.low, zigzag, pivots);
    }
    
    for pole in zigzag {
        let value = if *mode == 2. {
            pole.value
        } else {
            pole.edge.value() as c_float * if pole.segmented { 100. } else { 1. }
        };
        *pfOUT.offset(pole.index as isize) = value;
    }
}

// mode=1（默认值), 中枢位置(-2,2), mode=2，中枢高, mode=3，中枢低, mode=4, 买卖点
pub unsafe extern "C" fn pivot(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let market = Market::new(DataLen as usize, pfINa_high, pfINb_low);
    let (_zigzag, signals, pivots) = market.zigzag();
    
    if *mode == 4. {
        for (index, signal) in signals.iter() {
            *pfOUT.offset(*index as isize) = *signal as i32 as c_float;
        }
        return;
    }
    for pivot in pivots {
        if *mode == 2. {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.high() as c_float;
            }
        } else if *mode == 3. {
            for i in pivot.start()..=pivot.end() {
                *pfOUT.offset(i as isize) = pivot.low() as c_float;
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