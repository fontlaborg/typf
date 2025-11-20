# TYPF Benchmark Summary

**Date**: 2025-11-20 13:52:52  
**Iterations**: 100  
**Success Rate**: 240/240

## ⚠️ Performance Regressions Detected

**46 backend(s)** are >10% slower than baseline:

| Backend | Text | Size | Baseline | Current | Slowdown |

|---------|------|------|----------|---------|----------|
| none + JSON | mixd | 64.0px | 0.112ms | 0.155ms | +37.7% |
| none + coregraphics | mixd | 16.0px | 0.140ms | 0.155ms | +10.4% |
| none + coregraphics | mixd | 32.0px | 0.134ms | 0.161ms | +20.1% |
| none + coregraphics | mixd | 64.0px | 0.143ms | 0.213ms | +49.0% |
| none + skia | arab | 64.0px | 0.936ms | 1.083ms | +15.6% |
| HarfBuzz + JSON | latn | 32.0px | 0.045ms | 0.071ms | +55.2% |
| HarfBuzz + JSON | latn | 64.0px | 0.045ms | 0.072ms | +62.3% |
| HarfBuzz + JSON | latn | 128.0px | 0.045ms | 0.087ms | +96.3% |
| HarfBuzz + JSON | arab | 16.0px | 0.057ms | 0.083ms | +47.0% |
| HarfBuzz + JSON | arab | 32.0px | 0.053ms | 0.078ms | +48.3% |

*...and 36 more (see benchmark_report.json)*


## Detailed Performance (Ops/sec)

| Backend | Text | Size | Ops/sec |

|:---|:---|:---:|---:|
| HarfBuzz + JSON | arab | 16px | 12,028.6 |
| HarfBuzz + JSON | arab | 32px | 12,739.3 |
| HarfBuzz + JSON | arab | 64px | 12,866.1 |
| HarfBuzz + JSON | arab | 128px | 13,619.0 |
| HarfBuzz + JSON | latn | 16px | 21,903.0 |
| HarfBuzz + JSON | latn | 32px | 14,184.1 |
| HarfBuzz + JSON | latn | 64px | 13,810.7 |
| HarfBuzz + JSON | latn | 128px | 11,443.1 |
| HarfBuzz + JSON | mixd | 16px | 6,229.8 |
| HarfBuzz + JSON | mixd | 32px | 7,661.2 |
| HarfBuzz + JSON | mixd | 64px | 8,292.7 |
| HarfBuzz + JSON | mixd | 128px | 4,415.2 |
| HarfBuzz + coregraphics | arab | 16px | 3,707.9 |
| HarfBuzz + coregraphics | arab | 32px | 5,224.3 |
| HarfBuzz + coregraphics | arab | 64px | 4,441.6 |
| HarfBuzz + coregraphics | arab | 128px | 3,286.5 |
| HarfBuzz + coregraphics | latn | 16px | 1,426.6 |
| HarfBuzz + coregraphics | latn | 32px | 1,371.5 |
| HarfBuzz + coregraphics | latn | 64px | 1,333.1 |
| HarfBuzz + coregraphics | latn | 128px | 1,205.6 |
| HarfBuzz + coregraphics | mixd | 16px | 6,290.7 |
| HarfBuzz + coregraphics | mixd | 32px | 4,975.8 |
| HarfBuzz + coregraphics | mixd | 64px | 5,202.1 |
| HarfBuzz + coregraphics | mixd | 128px | 4,128.2 |
| HarfBuzz + orge | arab | 16px | 5,536.4 |
| HarfBuzz + orge | arab | 32px | 2,853.7 |
| HarfBuzz + orge | arab | 64px | 1,288.3 |
| HarfBuzz + orge | arab | 128px | 448.7 |
| HarfBuzz + orge | latn | 16px | 3,538.1 |
| HarfBuzz + orge | latn | 32px | 1,832.7 |
| HarfBuzz + orge | latn | 64px | 731.2 |
| HarfBuzz + orge | latn | 128px | 238.2 |
| HarfBuzz + orge | mixd | 16px | 4,312.3 |
| HarfBuzz + orge | mixd | 32px | 2,690.2 |
| HarfBuzz + orge | mixd | 64px | 1,339.5 |
| HarfBuzz + orge | mixd | 128px | 476.0 |
| HarfBuzz + skia | arab | 16px | 3,744.3 |
| HarfBuzz + skia | arab | 32px | 2,176.1 |
| HarfBuzz + skia | arab | 64px | 1,243.3 |
| HarfBuzz + skia | arab | 128px | 555.9 |
| HarfBuzz + skia | latn | 16px | 2,525.6 |
| HarfBuzz + skia | latn | 32px | 1,390.3 |
| HarfBuzz + skia | latn | 64px | 630.3 |
| HarfBuzz + skia | latn | 128px | 264.7 |
| HarfBuzz + skia | mixd | 16px | 3,567.0 |
| HarfBuzz + skia | mixd | 32px | 2,354.4 |
| HarfBuzz + skia | mixd | 64px | 1,367.1 |
| HarfBuzz + skia | mixd | 128px | 624.4 |
| HarfBuzz + zeno | arab | 16px | 3,117.2 |
| HarfBuzz + zeno | arab | 32px | 2,676.9 |
| HarfBuzz + zeno | arab | 64px | 1,936.4 |
| HarfBuzz + zeno | arab | 128px | 1,156.1 |
| HarfBuzz + zeno | latn | 16px | 2,044.6 |
| HarfBuzz + zeno | latn | 32px | 1,867.9 |
| HarfBuzz + zeno | latn | 64px | 1,449.0 |
| HarfBuzz + zeno | latn | 128px | 778.4 |
| HarfBuzz + zeno | mixd | 16px | 3,158.1 |
| HarfBuzz + zeno | mixd | 32px | 4,061.9 |
| HarfBuzz + zeno | mixd | 64px | 2,487.4 |
| HarfBuzz + zeno | mixd | 128px | 1,317.7 |
| ICU-HarfBuzz + JSON | arab | 16px | 9,327.7 |
| ICU-HarfBuzz + JSON | arab | 32px | 16,497.6 |
| ICU-HarfBuzz + JSON | arab | 64px | 10,450.3 |
| ICU-HarfBuzz + JSON | arab | 128px | 16,390.4 |
| ICU-HarfBuzz + JSON | latn | 16px | 9,018.0 |
| ICU-HarfBuzz + JSON | latn | 32px | 15,647.2 |
| ICU-HarfBuzz + JSON | latn | 64px | 18,629.8 |
| ICU-HarfBuzz + JSON | latn | 128px | 20,245.6 |
| ICU-HarfBuzz + JSON | mixd | 16px | 5,007.0 |
| ICU-HarfBuzz + JSON | mixd | 32px | 4,806.0 |
| ICU-HarfBuzz + JSON | mixd | 64px | 5,180.5 |
| ICU-HarfBuzz + JSON | mixd | 128px | 5,457.6 |
| ICU-HarfBuzz + coregraphics | arab | 16px | 3,259.3 |
| ICU-HarfBuzz + coregraphics | arab | 32px | 3,661.1 |
| ICU-HarfBuzz + coregraphics | arab | 64px | 4,568.6 |
| ICU-HarfBuzz + coregraphics | arab | 128px | 3,720.5 |
| ICU-HarfBuzz + coregraphics | latn | 16px | 1,231.6 |
| ICU-HarfBuzz + coregraphics | latn | 32px | 1,398.1 |
| ICU-HarfBuzz + coregraphics | latn | 64px | 1,254.5 |
| ICU-HarfBuzz + coregraphics | latn | 128px | 1,124.6 |
| ICU-HarfBuzz + coregraphics | mixd | 16px | 3,851.0 |
| ICU-HarfBuzz + coregraphics | mixd | 32px | 4,585.0 |
| ICU-HarfBuzz + coregraphics | mixd | 64px | 4,446.6 |
| ICU-HarfBuzz + coregraphics | mixd | 128px | 3,630.6 |
| ICU-HarfBuzz + orge | arab | 16px | 5,544.8 |
| ICU-HarfBuzz + orge | arab | 32px | 3,202.9 |
| ICU-HarfBuzz + orge | arab | 64px | 1,277.1 |
| ICU-HarfBuzz + orge | arab | 128px | 459.4 |
| ICU-HarfBuzz + orge | latn | 16px | 3,277.7 |
| ICU-HarfBuzz + orge | latn | 32px | 1,784.2 |
| ICU-HarfBuzz + orge | latn | 64px | 718.3 |
| ICU-HarfBuzz + orge | latn | 128px | 237.6 |
| ICU-HarfBuzz + orge | mixd | 16px | 2,533.5 |
| ICU-HarfBuzz + orge | mixd | 32px | 2,796.0 |
| ICU-HarfBuzz + orge | mixd | 64px | 1,305.0 |
| ICU-HarfBuzz + orge | mixd | 128px | 422.9 |
| ICU-HarfBuzz + skia | arab | 16px | 3,678.5 |
| ICU-HarfBuzz + skia | arab | 32px | 2,362.4 |
| ICU-HarfBuzz + skia | arab | 64px | 1,178.9 |
| ICU-HarfBuzz + skia | arab | 128px | 526.6 |
| ICU-HarfBuzz + skia | latn | 16px | 2,455.1 |
| ICU-HarfBuzz + skia | latn | 32px | 1,277.7 |
| ICU-HarfBuzz + skia | latn | 64px | 625.6 |
| ICU-HarfBuzz + skia | latn | 128px | 234.5 |
| ICU-HarfBuzz + skia | mixd | 16px | 3,943.7 |
| ICU-HarfBuzz + skia | mixd | 32px | 2,053.0 |
| ICU-HarfBuzz + skia | mixd | 64px | 1,398.8 |
| ICU-HarfBuzz + skia | mixd | 128px | 591.6 |
| ICU-HarfBuzz + zeno | arab | 16px | 2,765.9 |
| ICU-HarfBuzz + zeno | arab | 32px | 2,664.9 |
| ICU-HarfBuzz + zeno | arab | 64px | 2,030.9 |
| ICU-HarfBuzz + zeno | arab | 128px | 1,116.2 |
| ICU-HarfBuzz + zeno | latn | 16px | 2,250.4 |
| ICU-HarfBuzz + zeno | latn | 32px | 1,783.6 |
| ICU-HarfBuzz + zeno | latn | 64px | 1,010.9 |
| ICU-HarfBuzz + zeno | latn | 128px | 793.6 |
| ICU-HarfBuzz + zeno | mixd | 16px | 1,920.4 |
| ICU-HarfBuzz + zeno | mixd | 32px | 2,339.3 |
| ICU-HarfBuzz + zeno | mixd | 64px | 2,205.2 |
| ICU-HarfBuzz + zeno | mixd | 128px | 1,142.4 |
| coretext + JSON | arab | 16px | 24,625.7 |
| coretext + JSON | arab | 32px | 15,043.1 |
| coretext + JSON | arab | 64px | 24,668.3 |
| coretext + JSON | arab | 128px | 13,149.1 |
| coretext + JSON | latn | 16px | 27,973.7 |
| coretext + JSON | latn | 32px | 20,373.7 |
| coretext + JSON | latn | 64px | 27,710.4 |
| coretext + JSON | latn | 128px | 13,281.3 |
| coretext + JSON | mixd | 16px | 5,386.9 |
| coretext + JSON | mixd | 32px | 6,658.0 |
| coretext + JSON | mixd | 64px | 6,926.7 |
| coretext + JSON | mixd | 128px | 7,920.6 |
| coretext + coregraphics | arab | 16px | 3,334.4 |
| coretext + coregraphics | arab | 32px | 6,852.4 |
| coretext + coregraphics | arab | 64px | 5,221.1 |
| coretext + coregraphics | arab | 128px | 3,610.2 |
| coretext + coregraphics | latn | 16px | 1,431.0 |
| coretext + coregraphics | latn | 32px | 1,399.5 |
| coretext + coregraphics | latn | 64px | 1,439.2 |
| coretext + coregraphics | latn | 128px | 1,231.2 |
| coretext + coregraphics | mixd | 16px | 3,835.9 |
| coretext + coregraphics | mixd | 32px | 5,165.3 |
| coretext + coregraphics | mixd | 64px | 4,253.9 |
| coretext + coregraphics | mixd | 128px | 3,493.6 |
| coretext + orge | arab | 16px | 5,510.3 |
| coretext + orge | arab | 32px | 3,432.7 |
| coretext + orge | arab | 64px | 1,520.8 |
| coretext + orge | arab | 128px | 521.5 |
| coretext + orge | latn | 16px | 3,744.9 |
| coretext + orge | latn | 32px | 1,797.2 |
| coretext + orge | latn | 64px | 732.7 |
| coretext + orge | latn | 128px | 229.8 |
| coretext + orge | mixd | 16px | 3,781.2 |
| coretext + orge | mixd | 32px | 2,868.5 |
| coretext + orge | mixd | 64px | 1,347.0 |
| coretext + orge | mixd | 128px | 480.2 |
| coretext + skia | arab | 16px | 4,073.0 |
| coretext + skia | arab | 32px | 2,049.8 |
| coretext + skia | arab | 64px | 1,347.3 |
| coretext + skia | arab | 128px | 570.3 |
| coretext + skia | latn | 16px | 2,819.8 |
| coretext + skia | latn | 32px | 1,374.9 |
| coretext + skia | latn | 64px | 641.9 |
| coretext + skia | latn | 128px | 259.0 |
| coretext + skia | mixd | 16px | 3,003.1 |
| coretext + skia | mixd | 32px | 2,405.4 |
| coretext + skia | mixd | 64px | 1,344.0 |
| coretext + skia | mixd | 128px | 598.7 |
| coretext + zeno | arab | 16px | 2,567.5 |
| coretext + zeno | arab | 32px | 2,732.4 |
| coretext + zeno | arab | 64px | 2,220.8 |
| coretext + zeno | arab | 128px | 1,197.3 |
| coretext + zeno | latn | 16px | 2,184.3 |
| coretext + zeno | latn | 32px | 1,694.5 |
| coretext + zeno | latn | 64px | 1,491.0 |
| coretext + zeno | latn | 128px | 833.9 |
| coretext + zeno | mixd | 16px | 3,075.3 |
| coretext + zeno | mixd | 32px | 2,681.4 |
| coretext + zeno | mixd | 64px | 2,347.7 |
| coretext + zeno | mixd | 128px | 1,266.3 |
| none + JSON | arab | 16px | 22,621.0 |
| none + JSON | arab | 32px | 15,938.3 |
| none + JSON | arab | 64px | 21,817.2 |
| none + JSON | arab | 128px | 21,870.3 |
| none + JSON | latn | 16px | 23,546.0 |
| none + JSON | latn | 32px | 24,060.6 |
| none + JSON | latn | 64px | 23,516.7 |
| none + JSON | latn | 128px | 23,966.7 |
| none + JSON | mixd | 16px | 9,612.4 |
| none + JSON | mixd | 32px | 10,662.9 |
| none + JSON | mixd | 64px | 6,457.4 |
| none + JSON | mixd | 128px | 8,162.9 |
| none + coregraphics | arab | 16px | 4,413.1 |
| none + coregraphics | arab | 32px | 7,324.3 |
| none + coregraphics | arab | 64px | 5,974.2 |
| none + coregraphics | arab | 128px | 3,856.5 |
| none + coregraphics | latn | 16px | 1,498.9 |
| none + coregraphics | latn | 32px | 1,512.9 |
| none + coregraphics | latn | 64px | 1,456.0 |
| none + coregraphics | latn | 128px | 1,221.5 |
| none + coregraphics | mixd | 16px | 6,448.4 |
| none + coregraphics | mixd | 32px | 6,229.9 |
| none + coregraphics | mixd | 64px | 4,686.6 |
| none + coregraphics | mixd | 128px | 4,411.5 |
| none + orge | arab | 16px | 5,296.2 |
| none + orge | arab | 32px | 2,585.6 |
| none + orge | arab | 64px | 1,091.5 |
| none + orge | arab | 128px | 353.7 |
| none + orge | latn | 16px | 3,368.4 |
| none + orge | latn | 32px | 1,820.0 |
| none + orge | latn | 64px | 727.8 |
| none + orge | latn | 128px | 233.7 |
| none + orge | mixd | 16px | 4,455.1 |
| none + orge | mixd | 32px | 2,991.6 |
| none + orge | mixd | 64px | 1,270.6 |
| none + orge | mixd | 128px | 472.4 |
| none + skia | arab | 16px | 3,612.8 |
| none + skia | arab | 32px | 2,040.9 |
| none + skia | arab | 64px | 923.5 |
| none + skia | arab | 128px | 449.1 |
| none + skia | latn | 16px | 2,505.9 |
| none + skia | latn | 32px | 1,378.1 |
| none + skia | latn | 64px | 615.9 |
| none + skia | latn | 128px | 262.0 |
| none + skia | mixd | 16px | 3,589.6 |
| none + skia | mixd | 32px | 2,337.9 |
| none + skia | mixd | 64px | 1,486.0 |
| none + skia | mixd | 128px | 614.2 |
| none + zeno | arab | 16px | 3,153.3 |
| none + zeno | arab | 32px | 2,621.3 |
| none + zeno | arab | 64px | 1,933.4 |
| none + zeno | arab | 128px | 1,018.9 |
| none + zeno | latn | 16px | 2,180.3 |
| none + zeno | latn | 32px | 1,847.6 |
| none + zeno | latn | 64px | 1,453.2 |
| none + zeno | latn | 128px | 797.4 |
| none + zeno | mixd | 16px | 5,077.9 |
| none + zeno | mixd | 32px | 4,218.4 |
| none + zeno | mixd | 64px | 2,846.2 |
| none + zeno | mixd | 128px | 1,305.3 |

---
*Made by FontLab - https://www.fontlab.com/*
