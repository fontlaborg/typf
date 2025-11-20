# TYPF Benchmark Summary

**Date**: 2025-11-21 00:48:16  
**Iterations**: 100  
**Success Rate**: 240/240

## ⚠️ Performance Regressions Detected

**15 backend(s)** are >10% slower than baseline:

| Backend | Text | Size | Baseline | Current | Slowdown |

|---------|------|------|----------|---------|----------|
| HarfBuzz + JSON | arab | 64.0px | 0.059ms | 0.069ms | +17.1% |
| HarfBuzz + zeno | mixd | 16.0px | 0.196ms | 0.231ms | +17.9% |
| ICU-HarfBuzz + coregraphics | mixd | 16.0px | 0.240ms | 0.312ms | +29.9% |
| ICU-HarfBuzz + coregraphics | mixd | 32.0px | 0.212ms | 0.318ms | +50.2% |
| ICU-HarfBuzz + coregraphics | mixd | 64.0px | 0.364ms | 0.729ms | +100.7% |
| ICU-HarfBuzz + coregraphics | mixd | 128.0px | 0.428ms | 0.828ms | +93.4% |
| ICU-HarfBuzz + skia | latn | 16.0px | 0.396ms | 0.538ms | +35.8% |
| ICU-HarfBuzz + skia | latn | 32.0px | 0.738ms | 0.823ms | +11.6% |
| ICU-HarfBuzz + skia | arab | 32.0px | 0.448ms | 0.611ms | +36.3% |
| ICU-HarfBuzz + skia | mixd | 16.0px | 0.306ms | 0.401ms | +31.0% |

*...and 5 more (see benchmark_report.json)*


## Detailed Performance (Ops/sec)

| Backend | Text | Size | Ops/sec |

|:---|:---|:---:|---:|
| HarfBuzz + JSON | arab | 16px | 18,568.7 |
| HarfBuzz + JSON | arab | 32px | 17,968.2 |
| HarfBuzz + JSON | arab | 64px | 14,518.3 |
| HarfBuzz + JSON | arab | 128px | 18,692.0 |
| HarfBuzz + JSON | latn | 16px | 23,596.7 |
| HarfBuzz + JSON | latn | 32px | 23,136.8 |
| HarfBuzz + JSON | latn | 64px | 21,330.7 |
| HarfBuzz + JSON | latn | 128px | 23,042.1 |
| HarfBuzz + JSON | mixd | 16px | 10,912.0 |
| HarfBuzz + JSON | mixd | 32px | 9,158.6 |
| HarfBuzz + JSON | mixd | 64px | 10,597.3 |
| HarfBuzz + JSON | mixd | 128px | 8,947.6 |
| HarfBuzz + coregraphics | arab | 16px | 3,122.6 |
| HarfBuzz + coregraphics | arab | 32px | 6,350.3 |
| HarfBuzz + coregraphics | arab | 64px | 4,795.3 |
| HarfBuzz + coregraphics | arab | 128px | 3,283.7 |
| HarfBuzz + coregraphics | latn | 16px | 1,388.8 |
| HarfBuzz + coregraphics | latn | 32px | 1,395.2 |
| HarfBuzz + coregraphics | latn | 64px | 1,345.8 |
| HarfBuzz + coregraphics | latn | 128px | 1,104.3 |
| HarfBuzz + coregraphics | mixd | 16px | 5,805.7 |
| HarfBuzz + coregraphics | mixd | 32px | 6,160.0 |
| HarfBuzz + coregraphics | mixd | 64px | 5,424.3 |
| HarfBuzz + coregraphics | mixd | 128px | 4,149.3 |
| HarfBuzz + orge | arab | 16px | 5,114.8 |
| HarfBuzz + orge | arab | 32px | 2,995.2 |
| HarfBuzz + orge | arab | 64px | 1,286.5 |
| HarfBuzz + orge | arab | 128px | 434.5 |
| HarfBuzz + orge | latn | 16px | 3,610.7 |
| HarfBuzz + orge | latn | 32px | 1,801.6 |
| HarfBuzz + orge | latn | 64px | 686.8 |
| HarfBuzz + orge | latn | 128px | 228.6 |
| HarfBuzz + orge | mixd | 16px | 4,584.2 |
| HarfBuzz + orge | mixd | 32px | 3,002.7 |
| HarfBuzz + orge | mixd | 64px | 1,291.8 |
| HarfBuzz + orge | mixd | 128px | 440.7 |
| HarfBuzz + skia | arab | 16px | 3,464.6 |
| HarfBuzz + skia | arab | 32px | 2,204.2 |
| HarfBuzz + skia | arab | 64px | 1,202.7 |
| HarfBuzz + skia | arab | 128px | 516.8 |
| HarfBuzz + skia | latn | 16px | 2,514.0 |
| HarfBuzz + skia | latn | 32px | 1,366.4 |
| HarfBuzz + skia | latn | 64px | 586.4 |
| HarfBuzz + skia | latn | 128px | 251.3 |
| HarfBuzz + skia | mixd | 16px | 3,418.2 |
| HarfBuzz + skia | mixd | 32px | 2,267.7 |
| HarfBuzz + skia | mixd | 64px | 1,303.9 |
| HarfBuzz + skia | mixd | 128px | 587.8 |
| HarfBuzz + zeno | arab | 16px | 2,728.8 |
| HarfBuzz + zeno | arab | 32px | 2,500.1 |
| HarfBuzz + zeno | arab | 64px | 1,906.1 |
| HarfBuzz + zeno | arab | 128px | 1,081.7 |
| HarfBuzz + zeno | latn | 16px | 2,113.4 |
| HarfBuzz + zeno | latn | 32px | 1,862.4 |
| HarfBuzz + zeno | latn | 64px | 1,444.1 |
| HarfBuzz + zeno | latn | 128px | 762.8 |
| HarfBuzz + zeno | mixd | 16px | 4,325.7 |
| HarfBuzz + zeno | mixd | 32px | 4,033.5 |
| HarfBuzz + zeno | mixd | 64px | 2,609.2 |
| HarfBuzz + zeno | mixd | 128px | 1,200.9 |
| ICU-HarfBuzz + JSON | arab | 16px | 16,565.7 |
| ICU-HarfBuzz + JSON | arab | 32px | 16,749.8 |
| ICU-HarfBuzz + JSON | arab | 64px | 17,101.6 |
| ICU-HarfBuzz + JSON | arab | 128px | 12,872.1 |
| ICU-HarfBuzz + JSON | latn | 16px | 22,627.6 |
| ICU-HarfBuzz + JSON | latn | 32px | 22,599.1 |
| ICU-HarfBuzz + JSON | latn | 64px | 17,860.5 |
| ICU-HarfBuzz + JSON | latn | 128px | 20,236.3 |
| ICU-HarfBuzz + JSON | mixd | 16px | 9,120.5 |
| ICU-HarfBuzz + JSON | mixd | 32px | 9,174.0 |
| ICU-HarfBuzz + JSON | mixd | 64px | 10,235.3 |
| ICU-HarfBuzz + JSON | mixd | 128px | 8,950.3 |
| ICU-HarfBuzz + coregraphics | arab | 16px | 3,610.1 |
| ICU-HarfBuzz + coregraphics | arab | 32px | 5,727.8 |
| ICU-HarfBuzz + coregraphics | arab | 64px | 4,858.5 |
| ICU-HarfBuzz + coregraphics | arab | 128px | 3,184.6 |
| ICU-HarfBuzz + coregraphics | latn | 16px | 1,340.8 |
| ICU-HarfBuzz + coregraphics | latn | 32px | 1,343.5 |
| ICU-HarfBuzz + coregraphics | latn | 64px | 1,306.4 |
| ICU-HarfBuzz + coregraphics | latn | 128px | 1,117.8 |
| ICU-HarfBuzz + coregraphics | mixd | 16px | 3,207.4 |
| ICU-HarfBuzz + coregraphics | mixd | 32px | 3,144.0 |
| ICU-HarfBuzz + coregraphics | mixd | 64px | 1,370.9 |
| ICU-HarfBuzz + coregraphics | mixd | 128px | 1,207.0 |
| ICU-HarfBuzz + orge | arab | 16px | 5,062.1 |
| ICU-HarfBuzz + orge | arab | 32px | 2,683.2 |
| ICU-HarfBuzz + orge | arab | 64px | 1,191.3 |
| ICU-HarfBuzz + orge | arab | 128px | 431.8 |
| ICU-HarfBuzz + orge | latn | 16px | 3,580.8 |
| ICU-HarfBuzz + orge | latn | 32px | 1,833.6 |
| ICU-HarfBuzz + orge | latn | 64px | 680.8 |
| ICU-HarfBuzz + orge | latn | 128px | 225.6 |
| ICU-HarfBuzz + orge | mixd | 16px | 4,072.1 |
| ICU-HarfBuzz + orge | mixd | 32px | 2,960.9 |
| ICU-HarfBuzz + orge | mixd | 64px | 1,194.7 |
| ICU-HarfBuzz + orge | mixd | 128px | 436.9 |
| ICU-HarfBuzz + skia | arab | 16px | 3,050.1 |
| ICU-HarfBuzz + skia | arab | 32px | 1,637.0 |
| ICU-HarfBuzz + skia | arab | 64px | 1,060.3 |
| ICU-HarfBuzz + skia | arab | 128px | 501.4 |
| ICU-HarfBuzz + skia | latn | 16px | 1,857.2 |
| ICU-HarfBuzz + skia | latn | 32px | 1,214.6 |
| ICU-HarfBuzz + skia | latn | 64px | 585.4 |
| ICU-HarfBuzz + skia | latn | 128px | 248.3 |
| ICU-HarfBuzz + skia | mixd | 16px | 2,491.3 |
| ICU-HarfBuzz + skia | mixd | 32px | 1,316.0 |
| ICU-HarfBuzz + skia | mixd | 64px | 973.6 |
| ICU-HarfBuzz + skia | mixd | 128px | 562.7 |
| ICU-HarfBuzz + zeno | arab | 16px | 2,759.4 |
| ICU-HarfBuzz + zeno | arab | 32px | 2,438.2 |
| ICU-HarfBuzz + zeno | arab | 64px | 1,877.9 |
| ICU-HarfBuzz + zeno | arab | 128px | 1,047.4 |
| ICU-HarfBuzz + zeno | latn | 16px | 2,223.4 |
| ICU-HarfBuzz + zeno | latn | 32px | 1,842.1 |
| ICU-HarfBuzz + zeno | latn | 64px | 1,319.8 |
| ICU-HarfBuzz + zeno | latn | 128px | 762.1 |
| ICU-HarfBuzz + zeno | mixd | 16px | 3,269.1 |
| ICU-HarfBuzz + zeno | mixd | 32px | 2,039.0 |
| ICU-HarfBuzz + zeno | mixd | 64px | 1,305.6 |
| ICU-HarfBuzz + zeno | mixd | 128px | 553.7 |
| coretext + JSON | arab | 16px | 26,675.3 |
| coretext + JSON | arab | 32px | 24,849.4 |
| coretext + JSON | arab | 64px | 26,592.2 |
| coretext + JSON | arab | 128px | 25,478.8 |
| coretext + JSON | latn | 16px | 29,264.7 |
| coretext + JSON | latn | 32px | 27,387.6 |
| coretext + JSON | latn | 64px | 26,231.2 |
| coretext + JSON | latn | 128px | 27,897.2 |
| coretext + JSON | mixd | 16px | 13,100.7 |
| coretext + JSON | mixd | 32px | 10,625.2 |
| coretext + JSON | mixd | 64px | 12,862.4 |
| coretext + JSON | mixd | 128px | 11,046.8 |
| coretext + coregraphics | arab | 16px | 3,771.2 |
| coretext + coregraphics | arab | 32px | 7,452.6 |
| coretext + coregraphics | arab | 64px | 5,828.3 |
| coretext + coregraphics | arab | 128px | 3,515.3 |
| coretext + coregraphics | latn | 16px | 1,350.9 |
| coretext + coregraphics | latn | 32px | 1,386.3 |
| coretext + coregraphics | latn | 64px | 1,364.4 |
| coretext + coregraphics | latn | 128px | 1,124.5 |
| coretext + coregraphics | mixd | 16px | 5,282.2 |
| coretext + coregraphics | mixd | 32px | 5,611.9 |
| coretext + coregraphics | mixd | 64px | 5,676.6 |
| coretext + coregraphics | mixd | 128px | 4,216.3 |
| coretext + orge | arab | 16px | 5,869.1 |
| coretext + orge | arab | 32px | 3,346.8 |
| coretext + orge | arab | 64px | 1,379.7 |
| coretext + orge | arab | 128px | 496.0 |
| coretext + orge | latn | 16px | 3,763.6 |
| coretext + orge | latn | 32px | 1,856.6 |
| coretext + orge | latn | 64px | 701.1 |
| coretext + orge | latn | 128px | 223.8 |
| coretext + orge | mixd | 16px | 4,985.7 |
| coretext + orge | mixd | 32px | 2,745.9 |
| coretext + orge | mixd | 64px | 1,285.5 |
| coretext + orge | mixd | 128px | 469.8 |
| coretext + skia | arab | 16px | 3,818.4 |
| coretext + skia | arab | 32px | 2,322.0 |
| coretext + skia | arab | 64px | 1,258.2 |
| coretext + skia | arab | 128px | 560.2 |
| coretext + skia | latn | 16px | 2,578.2 |
| coretext + skia | latn | 32px | 1,342.0 |
| coretext + skia | latn | 64px | 581.3 |
| coretext + skia | latn | 128px | 247.4 |
| coretext + skia | mixd | 16px | 3,480.3 |
| coretext + skia | mixd | 32px | 2,396.5 |
| coretext + skia | mixd | 64px | 1,422.4 |
| coretext + skia | mixd | 128px | 563.5 |
| coretext + zeno | arab | 16px | 3,153.9 |
| coretext + zeno | arab | 32px | 2,794.6 |
| coretext + zeno | arab | 64px | 2,091.8 |
| coretext + zeno | arab | 128px | 1,206.7 |
| coretext + zeno | latn | 16px | 2,115.2 |
| coretext + zeno | latn | 32px | 1,867.4 |
| coretext + zeno | latn | 64px | 1,366.0 |
| coretext + zeno | latn | 128px | 744.9 |
| coretext + zeno | mixd | 16px | 4,299.9 |
| coretext + zeno | mixd | 32px | 3,371.0 |
| coretext + zeno | mixd | 64px | 2,514.9 |
| coretext + zeno | mixd | 128px | 1,216.6 |
| none + JSON | arab | 16px | 20,336.6 |
| none + JSON | arab | 32px | 20,997.0 |
| none + JSON | arab | 64px | 23,670.5 |
| none + JSON | arab | 128px | 21,831.9 |
| none + JSON | latn | 16px | 23,755.1 |
| none + JSON | latn | 32px | 26,103.4 |
| none + JSON | latn | 64px | 26,430.8 |
| none + JSON | latn | 128px | 26,287.8 |
| none + JSON | mixd | 16px | 10,987.3 |
| none + JSON | mixd | 32px | 12,217.9 |
| none + JSON | mixd | 64px | 11,616.8 |
| none + JSON | mixd | 128px | 12,848.8 |
| none + coregraphics | arab | 16px | 4,549.3 |
| none + coregraphics | arab | 32px | 7,383.7 |
| none + coregraphics | arab | 64px | 6,401.1 |
| none + coregraphics | arab | 128px | 3,673.3 |
| none + coregraphics | latn | 16px | 1,368.8 |
| none + coregraphics | latn | 32px | 1,296.3 |
| none + coregraphics | latn | 64px | 1,333.4 |
| none + coregraphics | latn | 128px | 1,112.2 |
| none + coregraphics | mixd | 16px | 7,400.6 |
| none + coregraphics | mixd | 32px | 6,893.8 |
| none + coregraphics | mixd | 64px | 5,988.7 |
| none + coregraphics | mixd | 128px | 5,071.4 |
| none + orge | arab | 16px | 4,970.5 |
| none + orge | arab | 32px | 2,751.9 |
| none + orge | arab | 64px | 1,013.5 |
| none + orge | arab | 128px | 342.3 |
| none + orge | latn | 16px | 3,683.0 |
| none + orge | latn | 32px | 1,712.2 |
| none + orge | latn | 64px | 711.8 |
| none + orge | latn | 128px | 228.8 |
| none + orge | mixd | 16px | 5,765.3 |
| none + orge | mixd | 32px | 3,359.0 |
| none + orge | mixd | 64px | 1,308.8 |
| none + orge | mixd | 128px | 457.5 |
| none + skia | arab | 16px | 3,356.5 |
| none + skia | arab | 32px | 1,988.0 |
| none + skia | arab | 64px | 1,011.3 |
| none + skia | arab | 128px | 417.1 |
| none + skia | latn | 16px | 2,627.0 |
| none + skia | latn | 32px | 1,392.5 |
| none + skia | latn | 64px | 597.7 |
| none + skia | latn | 128px | 249.4 |
| none + skia | mixd | 16px | 3,878.4 |
| none + skia | mixd | 32px | 2,331.6 |
| none + skia | mixd | 64px | 1,375.0 |
| none + skia | mixd | 128px | 586.2 |
| none + zeno | arab | 16px | 2,946.3 |
| none + zeno | arab | 32px | 2,487.0 |
| none + zeno | arab | 64px | 1,871.6 |
| none + zeno | arab | 128px | 1,054.8 |
| none + zeno | latn | 16px | 2,064.7 |
| none + zeno | latn | 32px | 1,808.4 |
| none + zeno | latn | 64px | 1,384.3 |
| none + zeno | latn | 128px | 751.4 |
| none + zeno | mixd | 16px | 5,019.0 |
| none + zeno | mixd | 32px | 4,509.9 |
| none + zeno | mixd | 64px | 2,922.6 |
| none + zeno | mixd | 128px | 1,290.6 |

---
*Community project by FontLab - https://www.fontlab.org/*
