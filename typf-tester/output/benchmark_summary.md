# TYPF Benchmark Summary

**Date**: 2025-11-20 14:23:57  
**Iterations**: 100  
**Success Rate**: 240/240

## ⚠️ Performance Regressions Detected

**40 backend(s)** are >10% slower than baseline:

| Backend | Text | Size | Baseline | Current | Slowdown |

|---------|------|------|----------|---------|----------|
| none + JSON | arab | 16.0px | 0.044ms | 0.052ms | +18.0% |
| none + skia | arab | 32.0px | 0.490ms | 0.553ms | +12.8% |
| none + skia | mixd | 16.0px | 0.279ms | 0.381ms | +36.6% |
| none + skia | mixd | 64.0px | 0.673ms | 0.758ms | +12.7% |
| HarfBuzz + JSON | arab | 16.0px | 0.083ms | 0.257ms | +209.7% |
| HarfBuzz + JSON | arab | 32.0px | 0.078ms | 0.145ms | +84.7% |
| HarfBuzz + JSON | arab | 64.0px | 0.078ms | 0.095ms | +22.7% |
| HarfBuzz + JSON | mixd | 32.0px | 0.131ms | 0.163ms | +24.6% |
| HarfBuzz + JSON | mixd | 64.0px | 0.121ms | 0.208ms | +72.4% |
| HarfBuzz + coregraphics | mixd | 16.0px | 0.159ms | 0.296ms | +86.3% |

*...and 30 more (see benchmark_report.json)*


## Detailed Performance (Ops/sec)

| Backend | Text | Size | Ops/sec |

|:---|:---|:---:|---:|
| HarfBuzz + JSON | arab | 16px | 3,884.1 |
| HarfBuzz + JSON | arab | 32px | 6,898.6 |
| HarfBuzz + JSON | arab | 64px | 10,485.3 |
| HarfBuzz + JSON | arab | 128px | 16,169.0 |
| HarfBuzz + JSON | latn | 16px | 21,353.5 |
| HarfBuzz + JSON | latn | 32px | 19,362.8 |
| HarfBuzz + JSON | latn | 64px | 21,077.2 |
| HarfBuzz + JSON | latn | 128px | 11,665.0 |
| HarfBuzz + JSON | mixd | 16px | 8,039.2 |
| HarfBuzz + JSON | mixd | 32px | 6,148.1 |
| HarfBuzz + JSON | mixd | 64px | 4,810.2 |
| HarfBuzz + JSON | mixd | 128px | 6,164.2 |
| HarfBuzz + coregraphics | arab | 16px | 3,395.3 |
| HarfBuzz + coregraphics | arab | 32px | 6,427.0 |
| HarfBuzz + coregraphics | arab | 64px | 5,269.2 |
| HarfBuzz + coregraphics | arab | 128px | 3,669.0 |
| HarfBuzz + coregraphics | latn | 16px | 1,380.3 |
| HarfBuzz + coregraphics | latn | 32px | 1,471.4 |
| HarfBuzz + coregraphics | latn | 64px | 1,413.1 |
| HarfBuzz + coregraphics | latn | 128px | 1,205.6 |
| HarfBuzz + coregraphics | mixd | 16px | 3,376.0 |
| HarfBuzz + coregraphics | mixd | 32px | 1,348.7 |
| HarfBuzz + coregraphics | mixd | 64px | 1,238.4 |
| HarfBuzz + coregraphics | mixd | 128px | 1,888.4 |
| HarfBuzz + orge | arab | 16px | 5,602.4 |
| HarfBuzz + orge | arab | 32px | 3,220.8 |
| HarfBuzz + orge | arab | 64px | 1,348.4 |
| HarfBuzz + orge | arab | 128px | 445.3 |
| HarfBuzz + orge | latn | 16px | 3,593.0 |
| HarfBuzz + orge | latn | 32px | 1,747.5 |
| HarfBuzz + orge | latn | 64px | 704.0 |
| HarfBuzz + orge | latn | 128px | 230.1 |
| HarfBuzz + orge | mixd | 16px | 4,201.5 |
| HarfBuzz + orge | mixd | 32px | 2,626.4 |
| HarfBuzz + orge | mixd | 64px | 1,296.7 |
| HarfBuzz + orge | mixd | 128px | 472.7 |
| HarfBuzz + skia | arab | 16px | 3,695.3 |
| HarfBuzz + skia | arab | 32px | 2,366.3 |
| HarfBuzz + skia | arab | 64px | 1,242.1 |
| HarfBuzz + skia | arab | 128px | 536.8 |
| HarfBuzz + skia | latn | 16px | 2,687.7 |
| HarfBuzz + skia | latn | 32px | 1,405.6 |
| HarfBuzz + skia | latn | 64px | 583.7 |
| HarfBuzz + skia | latn | 128px | 242.8 |
| HarfBuzz + skia | mixd | 16px | 3,898.0 |
| HarfBuzz + skia | mixd | 32px | 2,533.1 |
| HarfBuzz + skia | mixd | 64px | 1,255.5 |
| HarfBuzz + skia | mixd | 128px | 388.4 |
| HarfBuzz + zeno | arab | 16px | 2,865.7 |
| HarfBuzz + zeno | arab | 32px | 2,616.8 |
| HarfBuzz + zeno | arab | 64px | 1,948.8 |
| HarfBuzz + zeno | arab | 128px | 1,142.8 |
| HarfBuzz + zeno | latn | 16px | 2,055.7 |
| HarfBuzz + zeno | latn | 32px | 1,835.5 |
| HarfBuzz + zeno | latn | 64px | 1,444.0 |
| HarfBuzz + zeno | latn | 128px | 822.5 |
| HarfBuzz + zeno | mixd | 16px | 2,931.8 |
| HarfBuzz + zeno | mixd | 32px | 2,509.8 |
| HarfBuzz + zeno | mixd | 64px | 2,285.6 |
| HarfBuzz + zeno | mixd | 128px | 1,300.0 |
| ICU-HarfBuzz + JSON | arab | 16px | 11,265.3 |
| ICU-HarfBuzz + JSON | arab | 32px | 13,468.1 |
| ICU-HarfBuzz + JSON | arab | 64px | 12,078.6 |
| ICU-HarfBuzz + JSON | arab | 128px | 18,349.0 |
| ICU-HarfBuzz + JSON | latn | 16px | 17,832.3 |
| ICU-HarfBuzz + JSON | latn | 32px | 16,824.5 |
| ICU-HarfBuzz + JSON | latn | 64px | 21,336.2 |
| ICU-HarfBuzz + JSON | latn | 128px | 21,756.3 |
| ICU-HarfBuzz + JSON | mixd | 16px | 5,244.4 |
| ICU-HarfBuzz + JSON | mixd | 32px | 7,128.3 |
| ICU-HarfBuzz + JSON | mixd | 64px | 7,151.3 |
| ICU-HarfBuzz + JSON | mixd | 128px | 6,060.2 |
| ICU-HarfBuzz + coregraphics | arab | 16px | 3,427.5 |
| ICU-HarfBuzz + coregraphics | arab | 32px | 3,986.9 |
| ICU-HarfBuzz + coregraphics | arab | 64px | 5,302.7 |
| ICU-HarfBuzz + coregraphics | arab | 128px | 3,775.0 |
| ICU-HarfBuzz + coregraphics | latn | 16px | 1,387.2 |
| ICU-HarfBuzz + coregraphics | latn | 32px | 1,444.0 |
| ICU-HarfBuzz + coregraphics | latn | 64px | 1,360.0 |
| ICU-HarfBuzz + coregraphics | latn | 128px | 1,216.6 |
| ICU-HarfBuzz + coregraphics | mixd | 16px | 5,223.1 |
| ICU-HarfBuzz + coregraphics | mixd | 32px | 4,481.1 |
| ICU-HarfBuzz + coregraphics | mixd | 64px | 4,362.3 |
| ICU-HarfBuzz + coregraphics | mixd | 128px | 3,636.0 |
| ICU-HarfBuzz + orge | arab | 16px | 4,645.1 |
| ICU-HarfBuzz + orge | arab | 32px | 3,037.8 |
| ICU-HarfBuzz + orge | arab | 64px | 1,221.2 |
| ICU-HarfBuzz + orge | arab | 128px | 452.6 |
| ICU-HarfBuzz + orge | latn | 16px | 3,385.4 |
| ICU-HarfBuzz + orge | latn | 32px | 1,866.9 |
| ICU-HarfBuzz + orge | latn | 64px | 728.8 |
| ICU-HarfBuzz + orge | latn | 128px | 231.9 |
| ICU-HarfBuzz + orge | mixd | 16px | 3,004.4 |
| ICU-HarfBuzz + orge | mixd | 32px | 2,795.8 |
| ICU-HarfBuzz + orge | mixd | 64px | 1,331.0 |
| ICU-HarfBuzz + orge | mixd | 128px | 464.4 |
| ICU-HarfBuzz + skia | arab | 16px | 3,653.2 |
| ICU-HarfBuzz + skia | arab | 32px | 1,914.3 |
| ICU-HarfBuzz + skia | arab | 64px | 1,229.5 |
| ICU-HarfBuzz + skia | arab | 128px | 524.8 |
| ICU-HarfBuzz + skia | latn | 16px | 2,573.0 |
| ICU-HarfBuzz + skia | latn | 32px | 1,376.3 |
| ICU-HarfBuzz + skia | latn | 64px | 635.3 |
| ICU-HarfBuzz + skia | latn | 128px | 261.9 |
| ICU-HarfBuzz + skia | mixd | 16px | 3,521.2 |
| ICU-HarfBuzz + skia | mixd | 32px | 2,323.1 |
| ICU-HarfBuzz + skia | mixd | 64px | 1,351.8 |
| ICU-HarfBuzz + skia | mixd | 128px | 602.5 |
| ICU-HarfBuzz + zeno | arab | 16px | 2,812.4 |
| ICU-HarfBuzz + zeno | arab | 32px | 2,654.0 |
| ICU-HarfBuzz + zeno | arab | 64px | 2,029.3 |
| ICU-HarfBuzz + zeno | arab | 128px | 1,124.4 |
| ICU-HarfBuzz + zeno | latn | 16px | 2,147.4 |
| ICU-HarfBuzz + zeno | latn | 32px | 1,882.9 |
| ICU-HarfBuzz + zeno | latn | 64px | 1,397.6 |
| ICU-HarfBuzz + zeno | latn | 128px | 797.7 |
| ICU-HarfBuzz + zeno | mixd | 16px | 4,068.3 |
| ICU-HarfBuzz + zeno | mixd | 32px | 3,582.3 |
| ICU-HarfBuzz + zeno | mixd | 64px | 2,756.0 |
| ICU-HarfBuzz + zeno | mixd | 128px | 1,314.8 |
| coretext + JSON | arab | 16px | 13,961.7 |
| coretext + JSON | arab | 32px | 12,983.8 |
| coretext + JSON | arab | 64px | 13,334.4 |
| coretext + JSON | arab | 128px | 11,844.2 |
| coretext + JSON | latn | 16px | 15,431.2 |
| coretext + JSON | latn | 32px | 16,174.4 |
| coretext + JSON | latn | 64px | 27,062.4 |
| coretext + JSON | latn | 128px | 16,833.7 |
| coretext + JSON | mixd | 16px | 4,566.1 |
| coretext + JSON | mixd | 32px | 7,779.4 |
| coretext + JSON | mixd | 64px | 4,834.1 |
| coretext + JSON | mixd | 128px | 7,136.3 |
| coretext + coregraphics | arab | 16px | 3,856.2 |
| coretext + coregraphics | arab | 32px | 6,311.8 |
| coretext + coregraphics | arab | 64px | 5,976.9 |
| coretext + coregraphics | arab | 128px | 3,979.1 |
| coretext + coregraphics | latn | 16px | 1,402.7 |
| coretext + coregraphics | latn | 32px | 1,484.3 |
| coretext + coregraphics | latn | 64px | 1,430.1 |
| coretext + coregraphics | latn | 128px | 1,229.2 |
| coretext + coregraphics | mixd | 16px | 3,185.1 |
| coretext + coregraphics | mixd | 32px | 3,483.3 |
| coretext + coregraphics | mixd | 64px | 3,772.4 |
| coretext + coregraphics | mixd | 128px | 2,538.5 |
| coretext + orge | arab | 16px | 6,362.4 |
| coretext + orge | arab | 32px | 3,464.5 |
| coretext + orge | arab | 64px | 1,484.8 |
| coretext + orge | arab | 128px | 456.5 |
| coretext + orge | latn | 16px | 3,204.0 |
| coretext + orge | latn | 32px | 1,807.8 |
| coretext + orge | latn | 64px | 697.2 |
| coretext + orge | latn | 128px | 231.4 |
| coretext + orge | mixd | 16px | 3,469.5 |
| coretext + orge | mixd | 32px | 2,099.4 |
| coretext + orge | mixd | 64px | 1,007.2 |
| coretext + orge | mixd | 128px | 440.7 |
| coretext + skia | arab | 16px | 4,096.5 |
| coretext + skia | arab | 32px | 2,455.1 |
| coretext + skia | arab | 64px | 1,279.2 |
| coretext + skia | arab | 128px | 581.4 |
| coretext + skia | latn | 16px | 2,486.3 |
| coretext + skia | latn | 32px | 1,423.0 |
| coretext + skia | latn | 64px | 361.8 |
| coretext + skia | latn | 128px | 262.0 |
| coretext + skia | mixd | 16px | 2,676.3 |
| coretext + skia | mixd | 32px | 2,341.9 |
| coretext + skia | mixd | 64px | 1,185.0 |
| coretext + skia | mixd | 128px | 603.5 |
| coretext + zeno | arab | 16px | 3,056.1 |
| coretext + zeno | arab | 32px | 2,735.1 |
| coretext + zeno | arab | 64px | 2,106.9 |
| coretext + zeno | arab | 128px | 1,258.5 |
| coretext + zeno | latn | 16px | 2,127.3 |
| coretext + zeno | latn | 32px | 1,945.6 |
| coretext + zeno | latn | 64px | 1,446.8 |
| coretext + zeno | latn | 128px | 788.6 |
| coretext + zeno | mixd | 16px | 3,790.6 |
| coretext + zeno | mixd | 32px | 3,570.8 |
| coretext + zeno | mixd | 64px | 2,376.0 |
| coretext + zeno | mixd | 128px | 1,216.7 |
| none + JSON | arab | 16px | 19,166.4 |
| none + JSON | arab | 32px | 21,803.9 |
| none + JSON | arab | 64px | 22,223.7 |
| none + JSON | arab | 128px | 22,038.8 |
| none + JSON | latn | 16px | 21,766.5 |
| none + JSON | latn | 32px | 22,554.7 |
| none + JSON | latn | 64px | 22,300.9 |
| none + JSON | latn | 128px | 22,824.3 |
| none + JSON | mixd | 16px | 8,791.5 |
| none + JSON | mixd | 32px | 10,474.0 |
| none + JSON | mixd | 64px | 11,544.0 |
| none + JSON | mixd | 128px | 12,060.4 |
| none + coregraphics | arab | 16px | 4,514.9 |
| none + coregraphics | arab | 32px | 7,551.7 |
| none + coregraphics | arab | 64px | 6,044.8 |
| none + coregraphics | arab | 128px | 3,762.8 |
| none + coregraphics | latn | 16px | 1,448.9 |
| none + coregraphics | latn | 32px | 1,520.4 |
| none + coregraphics | latn | 64px | 1,353.3 |
| none + coregraphics | latn | 128px | 1,248.0 |
| none + coregraphics | mixd | 16px | 6,038.4 |
| none + coregraphics | mixd | 32px | 6,042.0 |
| none + coregraphics | mixd | 64px | 6,178.3 |
| none + coregraphics | mixd | 128px | 4,858.0 |
| none + orge | arab | 16px | 5,380.6 |
| none + orge | arab | 32px | 2,805.2 |
| none + orge | arab | 64px | 1,050.5 |
| none + orge | arab | 128px | 349.4 |
| none + orge | latn | 16px | 3,730.9 |
| none + orge | latn | 32px | 1,896.8 |
| none + orge | latn | 64px | 720.2 |
| none + orge | latn | 128px | 232.7 |
| none + orge | mixd | 16px | 5,413.7 |
| none + orge | mixd | 32px | 3,224.1 |
| none + orge | mixd | 64px | 1,397.0 |
| none + orge | mixd | 128px | 473.4 |
| none + skia | arab | 16px | 3,358.9 |
| none + skia | arab | 32px | 1,809.0 |
| none + skia | arab | 64px | 1,077.3 |
| none + skia | arab | 128px | 438.4 |
| none + skia | latn | 16px | 2,572.5 |
| none + skia | latn | 32px | 1,420.5 |
| none + skia | latn | 64px | 634.3 |
| none + skia | latn | 128px | 266.3 |
| none + skia | mixd | 16px | 2,627.0 |
| none + skia | mixd | 32px | 2,367.5 |
| none + skia | mixd | 64px | 1,318.5 |
| none + skia | mixd | 128px | 612.2 |
| none + zeno | arab | 16px | 3,139.5 |
| none + zeno | arab | 32px | 2,673.9 |
| none + zeno | arab | 64px | 1,868.1 |
| none + zeno | arab | 128px | 1,062.7 |
| none + zeno | latn | 16px | 2,191.7 |
| none + zeno | latn | 32px | 1,851.4 |
| none + zeno | latn | 64px | 1,409.1 |
| none + zeno | latn | 128px | 802.8 |
| none + zeno | mixd | 16px | 5,003.4 |
| none + zeno | mixd | 32px | 4,357.7 |
| none + zeno | mixd | 64px | 3,019.5 |
| none + zeno | mixd | 128px | 1,352.9 |

---
*Community project by FontLab - https://www.fontlab.org/*
