# Rust 实现缠论(缠中说禅)的通达信插件

# 使用方法

```shell
rustup target add  i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
copy .\target\i686-pc-windows-msvc\release\rust_chan.dll C:\tdx\T0002\dlls
```

## 通达信公式
绑定到第一个dll，并编辑公式如下:

1、 缠论端点和买卖点公式，命名为 ZEN
```
FRAC:TDXDLL1(4, H, L, 1);

MH:=TDXDLL1(3, H, L, 1);
ML:=TDXDLL1(3, H, L, -1);

CH:IF(MH>0, MH, H);
CL:IF(ML>0, ML, L);
DUAN:TDXDLL1(5, FRAC, CH, CL);
BUY1S:TDXDLL1(10, FRAC, CH, CL);
BUY1B:TDXDLL1(9, FRAC, CH, CL);


一买:BUY1B==1 OR BUY1S==1;
一卖:BUY1B==-1 OR BUY1S==-1;
三买:BUY1B==3 OR BUY1S==3 OR BUY1B=23 OR BUY1S=23;
三卖:BUY1B==-3 OR BUY1S==-3 OR BUY1B=-23 OR BUY1S=-23;
二买:BUY1B==2 OR BUY1S==2 OR BUY1B=23 OR BUY1S=23;
二卖:BUY1B==-2 OR BUY1S==-2 OR BUY1B=-23 OR BUY1S=-23;

笔幅:IF(FRAC=-1, REF(CH, BARSLAST(FRAC=1)) - CL, IF(FRAC=1, CH - REF(CL, BARSLAST(FRAC=-1)), DRAWNULL));
笔天数:IF(FRAC=-1, BARSLAST(FRAC=1) + 1 , IF(FRAC=1, BARSLAST(FRAC=-1) + 1, DRAWNULL));
笔力度:IF(FRAC=1 OR FRAC=-1, 笔幅/笔天数*100, DRAWNULL);
笔K数:IF(REF(BARSLAST(FRAC=-1),1) < REF(BARSLAST(FRAC=1),1), COUNT(CL <> REF(CL, 1), BARSLAST(FRAC=-1)) + 1, COUNT(CH <> REF(CH, 1), BARSLAST(FRAC=1)) + 1);

段幅:IF(DUAN=-1, REF(CH, BARSLAST(DUAN=1)) - CL, IF(DUAN=1, CH - REF(L, BARSLAST(DUAN=-1)), DRAWNULL));
段天数:IF(DUAN=-1, BARSLAST(DUAN=1) + 1 , IF(DUAN=1, BARSLAST(DUAN=-1) + 1, DRAWNULL));
段力度:IF(DUAN=1 OR DUAN=-1, 段幅/段天数*100, DRAWNULL);
段K数:IF(REF(BARSLAST(DUAN=-1),1) < REF(BARSLAST(DUAN=1),1), COUNT(CL <> REF(CL, 1), BARSLAST(DUAN=-1)) + 1, COUNT(CH <> REF(CH, 1), BARSLAST(DUAN=1)) + 1);

前笔天数:=IF(FRAC=-1, REF(笔天数, REF(BARSLAST(FRAC=-1), BARSLAST(FRAC=1)) + BARSLAST(FRAC=1)), REF(笔天数, REF(BARSLAST(FRAC=1), BARSLAST(FRAC=-1)) + BARSLAST(FRAC=-1)));
前笔幅:=IF(FRAC=-1, REF(笔幅, REF(BARSLAST(FRAC=-1), BARSLAST(FRAC=1)) + BARSLAST(FRAC=1)), REF(笔幅, REF(BARSLAST(FRAC=1), BARSLAST(FRAC=-1)) + BARSLAST(FRAC=-1)));
前笔力度:=IF(FRAC=-1, REF(笔力度, REF(BARSLAST(FRAC=-1), BARSLAST(FRAC=1)) + BARSLAST(FRAC=1)), REF(笔力度, REF(BARSLAST(FRAC=1), BARSLAST(FRAC=-1)) + BARSLAST(FRAC=-1)));

前段天数:=IF(DUAN=-1, REF(段天数, REF(BARSLAST(DUAN=-1), BARSLAST(DUAN=1)) + BARSLAST(DUAN=1)), REF(段天数, REF(BARSLAST(DUAN=1), BARSLAST(DUAN=-1)) + BARSLAST(DUAN=-1)));
前段幅:=IF(DUAN=-1, REF(段幅, REF(BARSLAST(DUAN=-1), BARSLAST(DUAN=1)) + BARSLAST(DUAN=1)), REF(段幅, REF(BARSLAST(DUAN=1), BARSLAST(DUAN=-1)) + BARSLAST(DUAN=-1)));
前段力度:=IF(DUAN=-1, REF(段力度, REF(BARSLAST(DUAN=-1), BARSLAST(DUAN=1)) + BARSLAST(DUAN=1)), REF(段力度, REF(BARSLAST(DUAN=1), BARSLAST(DUAN=-1)) + BARSLAST(DUAN=-1)));

笔极量能:IF(FRAC=-1 OR FRAC=1, HHV(V, 5), DRAWNULL);
段极量能:IF(DUAN=-1 OR DUAN=1, HHV(V, 5), DRAWNULL);
前笔极量能:=IF(FRAC=-1, REF(笔极量能, REF(BARSLAST(FRAC=-1), BARSLAST(FRAC=1)) + BARSLAST(FRAC=1)), REF(笔极量能, REF(BARSLAST(FRAC=1), BARSLAST(FRAC=-1)) + BARSLAST(FRAC=-1)));
前段极量能:=IF(DUAN=-1, REF(段极量能, REF(BARSLAST(DUAN=-1), BARSLAST(DUAN=1)) + BARSLAST(DUAN=1)), REF(段极量能, REF(BARSLAST(DUAN=1), BARSLAST(DUAN=-1)) + BARSLAST(DUAN=-1)));

笔背离:笔天数 > 前笔天数 AND (笔幅 < 前笔幅 AND 笔力度 < 前笔力度);
段背离:段天数 > 前段天数 AND (段幅 < 前段幅 AND 段力度 < 前段力度);


BISE:=TDXDLL1(6, FRAC, CH, CL);

前中枢数量:=REF(COUNT(BISE=1, BARSLAST(ABS(DUAN)=1)), 1);
背驰比较点:=(ABS(DUAN)=1 OR BISE=1) AND 前中枢数量 > 0;

段起点:=REF(BARSLAST(ABS(DUAN)=1), 1) + 1;
前中枢起点:=REF(BARSLAST(BISE=1), 1) + 1;
前中枢终点:=BARSLAST(BISE=2);
向上:=DUAN=1 OR (DUAN!=-1 AND BARSLAST(DUAN=-1) < BARSLAST(DUAN=1));
前前中枢终点:=IF(前中枢数量=1, REF(BARSLAST(ABS(DUAN)=1), 1) + 1, REF(BARSLAST(BISE=2), 前中枢起点) + 前中枢起点),NODRAW;

小B幅:IF(背驰比较点, ABS(IF(向上, REF(CL, 前中枢终点), REF(CH, 前中枢终点)) - IF(向上, CH, CL)), DRAWNULL),NODRAW;
小B天数:IF(背驰比较点,  前中枢终点+1, DRAWNULL),NODRAW;
小A幅:IF(背驰比较点, ABS(IF(向上, REF(CL, 前前中枢终点), REF(CH, 前前中枢终点)) - IF(向上, REF(CH, 前中枢起点), REF(CL, 前中枢起点))) , DRAWNULL),NODRAW;
小A天数:IF(背驰比较点, 前前中枢终点 - 前中枢起点 + 1, DRAWNULL),NODRAW;
背驰:小B幅 > 0 AND 小A幅 > 0 AND 小B天数 > 小A天数 AND 小B幅 < 小A幅;
```

2、专家系统公式
```
{多头买入(买开)} ENTERLONG: ZEN.一买;
{多头卖出(卖平)} EXITLONG: ZEN.一卖;
{空头卖出(卖开)} ENTERSHORT: ZEN.三卖;
{空头买入(买平)} EXITSHORT: ZEN.三买;
```

3、主图绘图公式
```
DRAWICON(ZEN.二买, L, 5);
DRAWICON(ZEN.二卖, L, 4);

CH:=ZEN.CH;
CL:=ZEN.CL;
FRAC:=ZEN.FRAC;
DUAN1:=ZEN.DUAN;
BUY1S:ZEN.BUY1S,NODRAW;
BUY1B:ZEN.BUY1B,NODRAW;

笔背离:ZEN.笔背离,NODRAW;
段背离:ZEN.段背离,NODRAW;
背驰:ZEN.背驰,NODRAW;
XX:STICKLINE(CH <> H OR CL <> L, CL, CH, 4, -1), COLORWHITE;
DRAWKLINE(H, O, L, C);
笔幅:ZEN.笔幅,NODRAW;

笔天数:ZEN.笔天数,NODRAW;
笔K数:ZEN.笔K数,NODRAW;
笔力度:ZEN.笔力度,NODRAW;

DRAWLINE(FRAC=-1,CL,FRAC=+1,CH,0), DOTLINE, COLORYELLOW;
DRAWLINE(FRAC=+1,CH,FRAC=-1,CL,0), DOTLINE, COLORYELLOW;

BISE:=TDXDLL1(6, FRAC, CH, CL),NODRAW;
BIZG:=TDXDLL1(7, FRAC, CH, CL);
BIZD:=TDXDLL1(8, FRAC, CH, CL);


DRAWLINE(DUAN1=-1,CL,DUAN1=+1,CH,0), LINETHICK2, COLORFF8000;
DRAWLINE(DUAN1=+1,CH,DUAN1=-1,CL,0), LINETHICK2, COLORFF8000;

DRAWLINE(DUAN1=-1,CL,DUAN1=+2,CH,0), LINETHICK2, COLORLIBLUE;
DRAWLINE(DUAN1=+1,CH,DUAN1=-2,CL,0), LINETHICK2, COLORLIBLUE;

DUANZG1:=TDXDLL1(7,DUAN1,CH,CL);
DUANZD1:=TDXDLL1(8,DUAN1,CH,CL);
DUANSE1:=TDXDLL1(6,DUAN1,CH,CL),NODRAW;

NOTEXT_DDUANZG1:IF(DUANZG1>0,DUANZG1,DRAWNULL),COLORFF8000;
NOTEXT_DDUANZD1:IF(DUANZD1>0,DUANZD1,DRAWNULL),COLORFF8000;

DRAWICON(ZEN.笔背离, IF(FRAC=-1,H, L), 31);
DRAWICON(ZEN.段背离, IF(DUAN1=-1,H, L), 32);
DRAWICON(ZEN.背驰, IF(FRAC=-1,H,L), 8);
```
4、选股公式，周期内最近7天有2买
```
N=7;
B1B:=BARSLAST(ZEN.BUY1B=1);
B2B:=BARSLAST(ZEN.BUY1B=2);
B1B > B2B AND B2B < N;
```