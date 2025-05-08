// 通达信插件选股器
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
mod market;
use std::{ffi::CStr, fs::OpenOptions, io::Write, mem, os::raw::{c_char, c_float, c_int, c_long, c_short, c_uchar, c_ulong, c_ushort, c_void}, ptr::{null, null_mut}};
use market::{BiMode, Market, PivotFinder, PivotMode};

use encoding::{all::GB18030, Encoding};
macro_rules! log {
    ($format:tt, $($arg:expr),*) => {
        write_log_entry(format_args!($format, $($arg),*))
    };
}
pub fn write_log_entry(entry: std::fmt::Arguments) {
    let mut file = OpenOptions::new().append(true).create(true)
        .open("c:/tdx/picker.txt").expect("failed to open log file");
    file.write_fmt(entry).expect("failed to write to log");
}
pub const ASK_ALL: c_int = -1;

// NTime时间信息
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NTime {
    pub year: c_ushort,
    pub month: c_uchar,
    pub day: c_uchar,
    pub hour: c_uchar,
    pub minute: c_uchar,
    pub second: c_uchar,
}

// 分析数据
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct HISDAT {
    pub Time: NTime,            // 时间
    pub Open: c_float,          // 开盘价
    pub High: c_float,          // 最高价
    pub Low: c_float,           // 最低价
    pub Close: c_float,         // 收盘价
//    pub Amount: c_float,        // 成交金额
    pub union1: HISDAT_Union1,  // 成交金额或持仓量
    pub fVolume: c_float,       // 成交量
//    pub lYClose: c_long,        // 
    pub union2: HISDAT_Union2,  // 结算价或上涨下跌家数
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union HISDAT_Union1 {
    pub Amount: c_float,        // 成交金额
    pub VolInStock: c_ulong,    // 持仓量(期货有效)
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union HISDAT_Union2 {
    pub Settle: c_float,        // 结算价(期货有效)
    pub lYClose: c_long,        // 
    pub zd: HISDAT_ZD,          // 上涨下跌家数(指数有效)
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct HISDAT_ZD {
    pub up: c_ushort,           // 上涨家数(指数有效)
    pub down: c_ushort,         // 下跌家数(指数有效)
}

pub type LPHISDAT = *mut HISDAT;

// 行情数据(第二版)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct REPORTDAT2 {
    pub ItemNum: c_ulong,           // 采样点数
    pub Close: c_float,             // 前收盘价
    pub Open: c_float,              // 今开盘价
    pub Max: c_float,               // 最高价
    pub Min: c_float,               // 最低价
    pub Now: c_float,               // 现价
    pub RefreshNum: c_ulong,        // 刷新数
    pub Volume: c_ulong,            // 总手
    pub NowVol: c_ulong,            // 现手(总手差)
    pub Amount: c_float,            // 总成交金额
    pub Inside: c_ulong,            // 内盘
    pub Outside: c_ulong,           // 外盘
    pub TickDiff: c_float,          // 笔涨跌(价位差)
    pub InOutFlag: c_uchar,         // 内外盘标志 0:Buy 1:Sell 2:None
    pub CJBS: c_ulong,              // 成交笔数
    pub Jjjz: c_float,              // 基金净值
    pub Ggpv: REPORTDAT2_Ggpv,      // 个股数据
    // pub Other: REPORTDAT2_Other,     // 个股或指数数据
    pub ununsed: [c_char; 20],      // 备用
}
/*
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub union REPORTDAT2_Other {
    pub Ggpv: REPORTDAT2_Ggpv,      // 个股数据
    pub Zspv: REPORTDAT2_Zspv,      // 指数数据
}
*/
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct REPORTDAT2_Ggpv {
    pub Buyp: [c_float; 5],         // 五个叫买价
    pub Buyv: [c_ulong; 5],         // 对应五个叫买价的五个买盘
    pub Sellp: [c_float; 5],        // 五个叫卖价
    pub Sellv: [c_ulong; 5],        // 对应五个叫卖价的五个卖盘
}
/* 
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct REPORTDAT2_Zspv {
    pub LxValue: c_float,           // 领先指标
    pub Yield: c_float,             // 不含加权的指数
    pub UpHome: c_long,             // 上涨家数
    pub DownHome: c_long,           // 下跌家数
}
*/
pub type LPREPORTDAT2 = *mut REPORTDAT2;

// 品种基本数据
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct STOCKINFO {
    pub Name: [c_char; 9],           // 证券名称
    pub Unit: c_short,               // 交易单位
    pub VolBase: c_long,             // 量比的基量
    pub Fz: [c_short; 8],            // 开收市时间(4段)
    pub InitTimer: c_short,          // 初始化时间
    pub EndTimer: c_short,           // 收盘时间
    pub nDelayMin: c_short,          // 延时分钟数
    pub bBelongHS300: c_char,        // 是否属于沪深300板块
    pub bBelongHasKQZ: c_char,       // 是否属于含可转债板块
    pub nBelongRZRQ: c_char,         // 是否属于融资融券板块
    pub bQH: c_char,                 // 是否是期货品种
    pub bHKGP: c_char,               // 是否是港股品种
    pub QHVol_BaseRate: c_short,     // 期货的每手乘数
    pub MinPrice: c_float,           // 最小变动价位
    pub unused: [c_char; 1],         // 备用
    pub ActiveCapital: c_float,      // 流通股本
    pub J_start: c_long,             // 上市日期
    pub J_addr: c_short,             // 所属省份
    pub J_hy: c_short,               // 所属行业
    pub J_zgb: c_float,              // 总股本
    pub J_zjhhy: c_float,            // 证监会行业
    pub J_oldjly: c_float,           // 上年此期净利润
    pub J_oldzysy: c_float,          // 上年此期营业收入
    pub J_bg: c_float,               // B股
    pub J_hg: c_float,               // H股
    pub J_mgsy2: c_float,            // 季报每股收益 (财报中提供的每股收益,有争议的才填)
    pub J_zzc: c_float,              // 总资产(元)
    pub J_ldzc: c_float,             // 流动资产
    pub J_gdzc: c_float,             // 固定资产
    pub J_wxzc: c_float,             // 无形资产
    pub J_gdrs: c_float,             // 股东人数
    pub J_ldfz: c_float,             // 流动负债
    pub J_cqfz: c_float,             // 少数股东权益
    pub J_zbgjj: c_float,            // 资本公积金
    pub J_jzc: c_float,              // 股东权益(就是净资产)
    pub J_yysy: c_float,             // 营业收入
    pub J_yycb: c_float,             // 营业成本
    pub J_yszk: c_float,             // 应收帐款
    pub J_yyly: c_float,             // 营业利润
    pub J_tzsy: c_float,             // 投资收益
    pub J_jyxjl: c_float,            // 经营现金净流量
    pub J_zxjl: c_float,             // 总现金净流量
    pub J_ch: c_float,               // 存货
    pub J_lyze: c_float,             // 利益总额
    pub J_shly: c_float,             // 税后利益
    pub J_jly: c_float,              // 净利益
    pub J_wfply: c_float,            // 未分配利益
    pub J_mgjzc2: c_float,           // 季报每股净资产 (财报中提供的每股收益,有争议的才填)
    pub J_jyl: c_float,              // 净益率%
    pub J_mgwfp: c_float,            // 每股未分配
    pub J_mgsy: c_float,             // 每股收益(折算成全年的)
    pub J_mggjj: c_float,            // 每股公积金
    pub J_mgjzc: c_float,            // 每股净资产
    pub J_gdqyb: c_float,            // 股东权益比
}

pub type LPSTOCKINFO = *mut STOCKINFO;

// 股本总股本信息
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GBInfo {
    pub Zgb: c_float,
    pub Ltgb: c_float,
}

pub type LPGBINFO = *mut GBInfo;

// 股票涨跌停价格数据
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct TPPrice {
    pub Close: c_float,
    pub TPTop: c_float,
    pub TPBottom: c_float,
}

pub type LPTPPRICE = *mut TPPrice;

// 数据回调的类型
pub const PER_MIN5: c_short = 0;      // 5分钟数据
pub const PER_MIN15: c_short = 1;      // 15分钟数据
pub const PER_MIN30: c_short = 2;      // 30分钟数据
pub const PER_HOUR: c_short = 3;       // 1小时数据
pub const PER_DAY: c_short = 4;        // 日线数据
pub const PER_WEEK: c_short = 5;       // 周线数据
pub const PER_MONTH: c_short = 6;      // 月线数据
pub const PER_MIN1: c_short = 7;       // 1分钟数据
pub const PER_MINN: c_short = 8;       // 多分析数据(10)
pub const PER_DAYN: c_short = 9;       // 多天线数据(45)
pub const PER_SEASON: c_short = 10;    // 季线数据
pub const PER_YEAR: c_short = 11;      // 年线数据
pub const PER_SEC5: c_short = 12;      // 5秒线
pub const PER_SECN: c_short = 13;      // 多秒线(15)
pub const PER_PRD_DIY0: c_short = 14;  // DIY周期
pub const PER_PRD_DIY10: c_short = 24; // DIY周期

pub const REPORT_DAT2: c_short = 102;  // 行情数据(第二版)
pub const GBINFO_DAT: c_short = 103;   // 股本信息
pub const STKINFO_DAT: c_short = 105;  // 股票相关数据
pub const TPPRICE_DAT: c_short = 121;  // 涨跌停数据

// 参数信息的结构定义
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PluginPara {
    pub acParaName: [c_char; 14],  // 参数的中文名称
    pub nMin: c_int,               // 参数最小取值范围
    pub nMax: c_int,               // 参数最大取值范围
    pub nDefault: c_int,           // 系统推荐的缺省值
    pub nValue: c_int,             // 用户定义的值
}

pub type PLUGINPARAM = PluginPara;

// 插件信息
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PlugInfo {
    pub Name: [c_char; 50],        // 名称与版本
    pub Dy: [c_char; 30],          // 产地
    pub Author: [c_char; 30],      // 设计人
    pub Descript: [c_char; 100],   // 选股描述
    pub Period: [c_char; 30],      // 适应周期
    pub OtherInfo: [c_char; 300],
    pub ParamNum: c_short,         // 0<=参数个数<=4
    pub ParamInfo: [PLUGINPARAM; 4], // 参数信息
}

pub type LPPLUGIN = *mut PlugInfo;

// 回调函数类型
pub type PDATAIOFUNC = extern "C" fn(
    Code: *mut c_char,
    nSetCode: c_short,
    DataType: c_short,
    pData: *mut c_void,
    nDataNum: c_short,
    time1: NTime,
    time2: NTime,
    nTQ: c_uchar,
    unused: c_ulong,
) -> c_long;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Api {
    pub pFuncCallBack: Option<PDATAIOFUNC>,
}

static mut api: Api = Api { pFuncCallBack: None };

// 注册回调函数
#[no_mangle]
pub unsafe extern "C" fn RegisterDataInterface(pfn: PDATAIOFUNC) {
    api.pFuncCallBack = Some(pfn);
    log!("\n注册回调函数: {:?}", pfn);
}

unsafe fn strcpy(dst: *mut c_char, src: &str) {
    let gb2312_bytes = GB18030.encode(src, encoding::EncoderTrap::Strict).unwrap();
    let dest = dst as *mut u8;
    for (i, &byte) in gb2312_bytes.iter().chain([0].iter()).enumerate() {
        // if i >= info.Name.len() { break; } // 防止溢出
        *dest.add(i) = byte;
    }
}
// 得到版权信息
#[no_mangle]
pub unsafe extern "C" fn GetCopyRightInfo(info: LPPLUGIN) {
    strcpy((*info).Name.as_mut_ptr(), "缠论选股器1.0");
	strcpy((*info).Dy.as_mut_ptr(),"广州");
	strcpy((*info).Author.as_mut_ptr(),"chan2zen");
	strcpy((*info).Period.as_mut_ptr(),"中长期");
	strcpy((*info).Descript.as_mut_ptr(),"缠论买卖点");
	strcpy((*info).OtherInfo.as_mut_ptr(),"自定义买卖信号");
	//填写参数信息
	(*info).ParamNum = 1;
	strcpy((*info).ParamInfo[0].acParaName.as_mut_ptr(),"信号");
	(*info).ParamInfo[0].nMin=-3;
	(*info).ParamInfo[0].nMax=3;
	(*info).ParamInfo[0].nDefault=3;
}

// 按最近数据计算(nDataNum为ASK_ALL表示所有数据)
/**
1.Code为股票代码，如申请上证指数数据则赋值为999999
2.nSetCode为市场分类，0为深市，1为沪市
3.DataType为申请数据类型，缺省为日K线历史数据，如申请行情数据则赋值为REPORT_DAT2，其他相关类型参见OutStruct.h
4.pData为申请数据缓冲区，若为NULL且nDataNum为-1则函数返回历史数据个数
 */
#[no_mangle]
pub unsafe extern "C" fn InputInfoThenCalc1(
        Code: *mut c_char,
        nSetCode: c_short,
        Value: *mut c_int,
        DataType: c_short,
        nDataNum: c_short,
        nTQ: c_uchar,
        unused: c_ulong,
    ) -> c_int {
    let tmpTime = mem::zeroed();
    log!("\nreading data {}... time:{:?}\n", CStr::from_ptr(Code).to_str().unwrap(), mem::size_of_val(&tmpTime));
    /*
    let mut d : [HISDAT; 2000] = [mem::zeroed(); 2000];
    let pHistData = d.as_mut_ptr();
    */
    let size = mem::size_of::<HISDAT>() as isize;
    let mut array: Box<[HISDAT]> = vec![mem::zeroed(); nDataNum as usize].into_boxed_slice();
    let pHistData = array.as_mut_ptr();
    let mut readnum: c_long = 0;
    if let Some(pfn) = api.pFuncCallBack {
        readnum = pfn(
            Code,
            nSetCode,
            DataType,
            pHistData as *mut c_void,
            nDataNum,
            tmpTime,
            tmpTime,
            nTQ,
            unused,
        );
    }
    log!("\nread data num: {}, nDataNum: {}\n", readnum, nDataNum);
    if readnum < 10 { return 0; }
    let mut bytes = [0 as i8; 100];
    let dp = pHistData as *mut i8;
    for i in 0..100 {
        bytes[i] = *dp.offset(i as isize);
    }
    log!("\nbytes: {:?}\n", bytes);
    let sig = *Value.offset(0);
    let mut high = Vec::<f32>::with_capacity(readnum as usize);
    let mut low = Vec::<f32>::with_capacity(readnum as usize);
    let len = mem::size_of::<HISDAT>();

    let limit = readnum as isize;
    let mut ptr = pHistData;
    let mut count = 1isize;
    let end_ptr = pHistData.wrapping_offset(limit);
    while ptr != end_ptr {
        let h = (*ptr).High;
        let l = (*ptr).Low;
        log!("\ndata: {}-{}, count: {}, len: {}", h, l, count, len);
        count += 1;
        //high.push(h);
        //low.push(l);
        log!("\ndata pushed, {}", l);
        ptr = ptr.wrapping_offset(1);
        //if count >= limit  { break; }
    }
    let market = Market::new(readnum as usize, high.as_mut_ptr(), low.as_mut_ptr());
    let zigzag = market.zigzag(PivotMode::BI, true, true);
    if let Some(zigzag) = zigzag.bi_zigzag {
        log!("\nzigzag: {:?}", zigzag);
        if let Some(signals) = zigzag.signals {
            if let Some(key) = signals.keys().max() {
                
                let ok = readnum as usize - *key < 7;
                if !ok { return 0; }
                if let Some(signal) = signals.get(key) {
                    match signal {
                        market::Signal::BUY22 => {
                            if sig == 2 { return 1 } else { return 0 };
                        },
                        market::Signal::SELL22 => {
                            if sig == 2 { return 1 } else { return 0 };
                        },
                        market::Signal::BUY23 => {
                            if sig == 2 || sig == 3 { return 1 } else { return 0 };
                        },
                        market::Signal::SELL23 => {
                            if sig == -2 || sig == -3 { return 1 } else { return 0 };
                        },
                        _ => {if (*signal as i32) == sig { return 1 } else { return 0 }}
                    }
                }
            }
        }
    }
    0
}

// 选取区段计算
#[no_mangle]
pub unsafe extern "C" fn InputInfoThenCalc2(
        Code: *mut c_char,
        nSetCode: c_short,
        Value: *mut c_int,
        DataType: c_short,
        time1: NTime,
        time2: NTime,
        nTQ: c_uchar,
        unused: c_ulong,
    ) -> c_int {
    1
}