#![allow(non_snake_case)]
mod judger;
mod segmenter;
mod analyzer;
mod chart;
use std::os::raw::{c_int, c_float, c_ushort};

use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::judger::{Vertex};

#[allow(unused_macros)]
macro_rules! log {
    ($format:tt, $($arg:expr),*) => {
        write_log_entry(format_args!($format, $($arg),*))
    };
}
pub fn write_log_entry(entry: std::fmt::Arguments) {
    let mut file = OpenOptions::new().append(true).create(true)
        .open("c:/tdx/twist.txt").expect("failed to open log file");
    file.write_fmt(entry).expect("failed to write to log");
}

#[repr(C, packed(1))]
pub struct PluginTCalcFuncInfo {
    nFuncMark: c_ushort,
    pCallFunc: Option<PlugInFunc>,
}

type PlugInFunc = unsafe extern "C" fn(c_int, *mut c_float, *mut c_float, *mut c_float, *mut c_float);

/// # Safety
/// 根据顶底标识和极值分析中枢
pub unsafe extern "C" fn analysis(DataLen: c_int, pfOUT: *mut c_float, pfINa_frac: *mut c_float, pfINb_value: *mut c_float, mode: *mut c_float) {
    let mut vertexes = Vec::with_capacity(DataLen as usize / 5 );
    for i in 0..DataLen as usize {
        if let Ok(edge) = (*pfINa_frac.add(i) as i32).try_into() {
            let vertex = Vertex::new(i, edge, *pfINb_value.add(i));
            vertexes.push(vertex);
        }
    }
    if *mode > 1000.0 {
        *mode -= 1000.0;
        log!("\n\nlet values = {:?};\n\n", vertexes.iter().map(|v| v.value).collect::<Vec<_>>());
    }
    let trading_ranges = analyzer::Analyzer::analyze(&vertexes);
    if *mode == 104.0 { // 买卖点
        let signals = analyzer::Analyzer::signals(&trading_ranges, &vertexes);
        signals.iter().for_each(|signal| match signal {
            analyzer::Signal::Buy(value, index) => unsafe { *pfOUT.add(*index) = *value as f32 },
            analyzer::Signal::Sell(value, index) => unsafe { *pfOUT.add(*index) = -(*value as f32) },
        });
        return;
    }
    let mut last_end = None;
    for tr in trading_ranges {
        match *mode {
            101.0 => { // 中枢高
                for i in tr.start()..=tr.end() {
                    unsafe {
                        *pfOUT.add(i) = tr.get_resistance();
                    }
                }
            },
            102.0 => { // 中枢低
                for i in tr.start()..=tr.end() {
                    unsafe {
                        *pfOUT.add(i) = tr.get_support();
                    }
                }
            },
            103.0 => { // 起止位置
                let mut start = tr.start();
                if let Some(last_end) = last_end {
                    if last_end == tr.start() {
                        start += 1;
                    }
                }
                unsafe {
                    *pfOUT.add(start) = -2.0;
                    *pfOUT.add(tr.end()) = 2.0;
                }
                last_end = Some(tr.end());
            },
            _ => {},
        }
    }
}
/// # Safety
/// 计算端点位置, frac=1/-1 分别标识 Peak 和 Trough
pub unsafe extern "C" fn zigzag(DataLen: c_int, pfOUT: *mut c_float, pfINa_high: *mut c_float, pfINb_low: *mut c_float, mode: *mut c_float) {
    let judger = Box::new(judger::DefaultJudger::new(false, DataLen as usize / 3));
    let mut chart = chart::Chart::raw_with_judger(DataLen as usize, pfINa_high, pfINb_low, judger);
    if *mode == 1.0 {
        log!("\n\nlet mut highs = {:?};\nlet mut lows = {:?};\n\n", chart.highs, chart.lows);
    } 
    chart.merge();
    let vertexes = chart.get_vertexes();
    let mut segmenter = segmenter::Segmenter::with_capacity(vertexes.len() / 3);
    for vertex in vertexes {
        segmenter.step(&vertex);
        unsafe {
            *pfOUT.add(vertex.index) = vertex.edge.as_frac() * (1.0 + vertex.count as f32 * 10.0);
        }
    }
    for idx in segmenter.indexes() {
        unsafe {
            *pfOUT.add(idx) = *pfOUT.add(idx) * 2.0;
        }
    }
    let tip = segmenter.tip();
    if tip > 0 {
        unsafe {
            *pfOUT.add(tip) = *pfOUT.add(tip) * 3.0;
        }
    }
}
   
static mut G_CALC_FUNC_SETS: [PluginTCalcFuncInfo; 3] = [
    PluginTCalcFuncInfo {
        nFuncMark: 2,
        pCallFunc: Some(analysis),
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

/// # Safety
///
/// 通达信插件注册函数
#[allow(static_mut_refs)]
#[no_mangle]
pub unsafe extern "C" fn RegisterTdxFunc(pFun: *mut *mut PluginTCalcFuncInfo) -> c_int {
    if (*pFun).is_null() {
        *pFun = G_CALC_FUNC_SETS.as_mut_ptr();
        1
    } else {
        0
    }
}