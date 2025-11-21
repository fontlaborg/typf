# TYPF Benchmark Summary

**Date**: 2025-11-21 00:56:37  
**Iterations**: 100  
**Success Rate**: 240/240

## ⚠️ Performance Regressions Detected

**92 backend(s)** are >10% slower than baseline:

| Backend | Text | Size | Baseline | Current | Slowdown |

|---------|------|------|----------|---------|----------|
| none + JSON | latn | 32.0px | 0.038ms | 0.043ms | +13.0% |
| none + JSON | latn | 128.0px | 0.038ms | 0.044ms | +15.4% |
| none + JSON | mixd | 128.0px | 0.078ms | 0.089ms | +14.9% |
| none + orge | arab | 64.0px | 0.987ms | 1.420ms | +44.0% |
| none + orge | mixd | 16.0px | 0.173ms | 0.208ms | +20.2% |
| none + orge | mixd | 32.0px | 0.298ms | 0.487ms | +63.6% |
| none + orge | mixd | 64.0px | 0.764ms | 0.961ms | +25.8% |
| none + orge | mixd | 128.0px | 2.186ms | 2.908ms | +33.0% |
| none + coregraphics | arab | 64.0px | 0.156ms | 0.183ms | +17.4% |
| none + coregraphics | arab | 128.0px | 0.272ms | 0.314ms | +15.3% |

*...and 82 more (see benchmark_report.json)*


## Detailed Performance (Ops/sec)

| Backend | Text | Size | Ops/sec |

|:---|:---|:---:|---:|
| HarfBuzz + JSON | arab | 16px | 13,892.0 |
| HarfBuzz + JSON | arab | 32px | 11,711.3 |
| HarfBuzz + JSON | arab | 64px | 9,512.4 |
| HarfBuzz + JSON | arab | 128px | 11,630.3 |
| HarfBuzz + JSON | latn | 16px | 23,907.5 |
| HarfBuzz + JSON | latn | 32px | 15,153.1 |
| HarfBuzz + JSON | latn | 64px | 15,952.0 |
| HarfBuzz + JSON | latn | 128px | 16,249.6 |
| HarfBuzz + JSON | mixd | 16px | 6,927.5 |
| HarfBuzz + JSON | mixd | 32px | 7,215.2 |
| HarfBuzz + JSON | mixd | 64px | 4,533.1 |
| HarfBuzz + JSON | mixd | 128px | 6,754.3 |
| HarfBuzz + coregraphics | arab | 16px | 2,962.3 |
| HarfBuzz + coregraphics | arab | 32px | 4,320.1 |
| HarfBuzz + coregraphics | arab | 64px | 3,363.5 |
| HarfBuzz + coregraphics | arab | 128px | 2,646.1 |
| HarfBuzz + coregraphics | latn | 16px | 1,374.0 |
| HarfBuzz + coregraphics | latn | 32px | 1,436.4 |
| HarfBuzz + coregraphics | latn | 64px | 1,297.7 |
| HarfBuzz + coregraphics | latn | 128px | 599.7 |
| HarfBuzz + coregraphics | mixd | 16px | 2,202.8 |
| HarfBuzz + coregraphics | mixd | 32px | 3,229.0 |
| HarfBuzz + coregraphics | mixd | 64px | 2,759.3 |
| HarfBuzz + coregraphics | mixd | 128px | 2,502.4 |
| HarfBuzz + orge | arab | 16px | 2,709.9 |
| HarfBuzz + orge | arab | 32px | 3,225.2 |
| HarfBuzz + orge | arab | 64px | 1,229.2 |
| HarfBuzz + orge | arab | 128px | 432.6 |
| HarfBuzz + orge | latn | 16px | 3,744.4 |
| HarfBuzz + orge | latn | 32px | 818.8 |
| HarfBuzz + orge | latn | 64px | 652.3 |
| HarfBuzz + orge | latn | 128px | 228.5 |
| HarfBuzz + orge | mixd | 16px | 2,685.2 |
| HarfBuzz + orge | mixd | 32px | 2,202.4 |
| HarfBuzz + orge | mixd | 64px | 1,226.1 |
| HarfBuzz + orge | mixd | 128px | 448.2 |
| HarfBuzz + skia | arab | 16px | 3,476.6 |
| HarfBuzz + skia | arab | 32px | 2,188.1 |
| HarfBuzz + skia | arab | 64px | 1,138.9 |
| HarfBuzz + skia | arab | 128px | 498.2 |
| HarfBuzz + skia | latn | 16px | 1,519.2 |
| HarfBuzz + skia | latn | 32px | 1,208.0 |
| HarfBuzz + skia | latn | 64px | 558.2 |
| HarfBuzz + skia | latn | 128px | 242.7 |
| HarfBuzz + skia | mixd | 16px | 2,203.8 |
| HarfBuzz + skia | mixd | 32px | 2,020.9 |
| HarfBuzz + skia | mixd | 64px | 1,212.5 |
| HarfBuzz + skia | mixd | 128px | 255.9 |
| HarfBuzz + zeno | arab | 16px | 2,805.9 |
| HarfBuzz + zeno | arab | 32px | 2,394.2 |
| HarfBuzz + zeno | arab | 64px | 1,677.7 |
| HarfBuzz + zeno | arab | 128px | 1,058.8 |
| HarfBuzz + zeno | latn | 16px | 1,641.1 |
| HarfBuzz + zeno | latn | 32px | 1,040.0 |
| HarfBuzz + zeno | latn | 64px | 900.0 |
| HarfBuzz + zeno | latn | 128px | 673.1 |
| HarfBuzz + zeno | mixd | 16px | 4,020.5 |
| HarfBuzz + zeno | mixd | 32px | 3,181.8 |
| HarfBuzz + zeno | mixd | 64px | 2,512.9 |
| HarfBuzz + zeno | mixd | 128px | 1,104.7 |
| ICU-HarfBuzz + JSON | arab | 16px | 9,130.3 |
| ICU-HarfBuzz + JSON | arab | 32px | 11,921.2 |
| ICU-HarfBuzz + JSON | arab | 64px | 12,197.8 |
| ICU-HarfBuzz + JSON | arab | 128px | 11,162.6 |
| ICU-HarfBuzz + JSON | latn | 16px | 22,623.6 |
| ICU-HarfBuzz + JSON | latn | 32px | 9,925.2 |
| ICU-HarfBuzz + JSON | latn | 64px | 10,408.9 |
| ICU-HarfBuzz + JSON | latn | 128px | 11,748.2 |
| ICU-HarfBuzz + JSON | mixd | 16px | 5,684.1 |
| ICU-HarfBuzz + JSON | mixd | 32px | 7,420.2 |
| ICU-HarfBuzz + JSON | mixd | 64px | 5,717.8 |
| ICU-HarfBuzz + JSON | mixd | 128px | 6,404.5 |
| ICU-HarfBuzz + coregraphics | arab | 16px | 1,948.9 |
| ICU-HarfBuzz + coregraphics | arab | 32px | 3,510.2 |
| ICU-HarfBuzz + coregraphics | arab | 64px | 2,207.9 |
| ICU-HarfBuzz + coregraphics | arab | 128px | 3,291.0 |
| ICU-HarfBuzz + coregraphics | latn | 16px | 1,457.5 |
| ICU-HarfBuzz + coregraphics | latn | 32px | 1,446.6 |
| ICU-HarfBuzz + coregraphics | latn | 64px | 1,305.5 |
| ICU-HarfBuzz + coregraphics | latn | 128px | 1,116.8 |
| ICU-HarfBuzz + coregraphics | mixd | 16px | 6,478.3 |
| ICU-HarfBuzz + coregraphics | mixd | 32px | 6,231.7 |
| ICU-HarfBuzz + coregraphics | mixd | 64px | 5,851.5 |
| ICU-HarfBuzz + coregraphics | mixd | 128px | 4,238.7 |
| ICU-HarfBuzz + orge | arab | 16px | 5,439.8 |
| ICU-HarfBuzz + orge | arab | 32px | 3,150.9 |
| ICU-HarfBuzz + orge | arab | 64px | 1,302.7 |
| ICU-HarfBuzz + orge | arab | 128px | 451.5 |
| ICU-HarfBuzz + orge | latn | 16px | 3,658.6 |
| ICU-HarfBuzz + orge | latn | 32px | 1,764.4 |
| ICU-HarfBuzz + orge | latn | 64px | 721.6 |
| ICU-HarfBuzz + orge | latn | 128px | 230.4 |
| ICU-HarfBuzz + orge | mixd | 16px | 4,345.6 |
| ICU-HarfBuzz + orge | mixd | 32px | 2,114.4 |
| ICU-HarfBuzz + orge | mixd | 64px | 1,260.5 |
| ICU-HarfBuzz + orge | mixd | 128px | 450.7 |
| ICU-HarfBuzz + skia | arab | 16px | 3,658.9 |
| ICU-HarfBuzz + skia | arab | 32px | 2,320.6 |
| ICU-HarfBuzz + skia | arab | 64px | 1,228.3 |
| ICU-HarfBuzz + skia | arab | 128px | 537.2 |
| ICU-HarfBuzz + skia | latn | 16px | 2,666.4 |
| ICU-HarfBuzz + skia | latn | 32px | 1,424.5 |
| ICU-HarfBuzz + skia | latn | 64px | 605.6 |
| ICU-HarfBuzz + skia | latn | 128px | 256.6 |
| ICU-HarfBuzz + skia | mixd | 16px | 3,670.7 |
| ICU-HarfBuzz + skia | mixd | 32px | 2,185.1 |
| ICU-HarfBuzz + skia | mixd | 64px | 1,187.9 |
| ICU-HarfBuzz + skia | mixd | 128px | 594.0 |
| ICU-HarfBuzz + zeno | arab | 16px | 2,662.5 |
| ICU-HarfBuzz + zeno | arab | 32px | 2,560.5 |
| ICU-HarfBuzz + zeno | arab | 64px | 1,956.9 |
| ICU-HarfBuzz + zeno | arab | 128px | 1,117.9 |
| ICU-HarfBuzz + zeno | latn | 16px | 2,189.8 |
| ICU-HarfBuzz + zeno | latn | 32px | 1,883.2 |
| ICU-HarfBuzz + zeno | latn | 64px | 1,368.6 |
| ICU-HarfBuzz + zeno | latn | 128px | 788.5 |
| ICU-HarfBuzz + zeno | mixd | 16px | 3,546.2 |
| ICU-HarfBuzz + zeno | mixd | 32px | 3,229.3 |
| ICU-HarfBuzz + zeno | mixd | 64px | 2,715.0 |
| ICU-HarfBuzz + zeno | mixd | 128px | 1,178.9 |
| coretext + JSON | arab | 16px | 13,302.1 |
| coretext + JSON | arab | 32px | 16,277.6 |
| coretext + JSON | arab | 64px | 15,552.7 |
| coretext + JSON | arab | 128px | 15,577.7 |
| coretext + JSON | latn | 16px | 23,208.4 |
| coretext + JSON | latn | 32px | 23,036.2 |
| coretext + JSON | latn | 64px | 8,260.7 |
| coretext + JSON | latn | 128px | 22,263.9 |
| coretext + JSON | mixd | 16px | 5,219.6 |
| coretext + JSON | mixd | 32px | 5,657.6 |
| coretext + JSON | mixd | 64px | 10,702.1 |
| coretext + JSON | mixd | 128px | 9,278.6 |
| coretext + coregraphics | arab | 16px | 3,999.1 |
| coretext + coregraphics | arab | 32px | 6,951.5 |
| coretext + coregraphics | arab | 64px | 5,628.0 |
| coretext + coregraphics | arab | 128px | 3,832.1 |
| coretext + coregraphics | latn | 16px | 1,274.5 |
| coretext + coregraphics | latn | 32px | 1,355.6 |
| coretext + coregraphics | latn | 64px | 1,295.1 |
| coretext + coregraphics | latn | 128px | 1,100.5 |
| coretext + coregraphics | mixd | 16px | 5,356.5 |
| coretext + coregraphics | mixd | 32px | 5,299.8 |
| coretext + coregraphics | mixd | 64px | 4,824.5 |
| coretext + coregraphics | mixd | 128px | 3,755.2 |
| coretext + orge | arab | 16px | 5,605.7 |
| coretext + orge | arab | 32px | 3,277.0 |
| coretext + orge | arab | 64px | 1,296.3 |
| coretext + orge | arab | 128px | 348.2 |
| coretext + orge | latn | 16px | 3,726.4 |
| coretext + orge | latn | 32px | 1,323.6 |
| coretext + orge | latn | 64px | 729.0 |
| coretext + orge | latn | 128px | 190.9 |
| coretext + orge | mixd | 16px | 3,855.6 |
| coretext + orge | mixd | 32px | 2,530.3 |
| coretext + orge | mixd | 64px | 1,232.6 |
| coretext + orge | mixd | 128px | 473.3 |
| coretext + skia | arab | 16px | 3,713.8 |
| coretext + skia | arab | 32px | 2,272.6 |
| coretext + skia | arab | 64px | 1,306.3 |
| coretext + skia | arab | 128px | 580.2 |
| coretext + skia | latn | 16px | 2,759.3 |
| coretext + skia | latn | 32px | 1,433.8 |
| coretext + skia | latn | 64px | 616.9 |
| coretext + skia | latn | 128px | 255.4 |
| coretext + skia | mixd | 16px | 2,521.2 |
| coretext + skia | mixd | 32px | 2,197.9 |
| coretext + skia | mixd | 64px | 1,336.1 |
| coretext + skia | mixd | 128px | 582.5 |
| coretext + zeno | arab | 16px | 3,237.3 |
| coretext + zeno | arab | 32px | 2,788.6 |
| coretext + zeno | arab | 64px | 2,139.7 |
| coretext + zeno | arab | 128px | 1,221.3 |
| coretext + zeno | latn | 16px | 2,185.3 |
| coretext + zeno | latn | 32px | 1,842.3 |
| coretext + zeno | latn | 64px | 1,296.6 |
| coretext + zeno | latn | 128px | 776.9 |
| coretext + zeno | mixd | 16px | 3,662.9 |
| coretext + zeno | mixd | 32px | 2,217.3 |
| coretext + zeno | mixd | 64px | 2,057.3 |
| coretext + zeno | mixd | 128px | 1,145.9 |
| none + JSON | arab | 16px | 23,563.8 |
| none + JSON | arab | 32px | 22,806.1 |
| none + JSON | arab | 64px | 23,508.0 |
| none + JSON | arab | 128px | 21,910.4 |
| none + JSON | latn | 16px | 23,584.0 |
| none + JSON | latn | 32px | 23,107.8 |
| none + JSON | latn | 64px | 24,660.9 |
| none + JSON | latn | 128px | 22,787.3 |
| none + JSON | mixd | 16px | 10,973.1 |
| none + JSON | mixd | 32px | 11,171.2 |
| none + JSON | mixd | 64px | 11,454.0 |
| none + JSON | mixd | 128px | 11,181.8 |
| none + coregraphics | arab | 16px | 4,267.5 |
| none + coregraphics | arab | 32px | 7,403.0 |
| none + coregraphics | arab | 64px | 5,454.6 |
| none + coregraphics | arab | 128px | 3,184.5 |
| none + coregraphics | latn | 16px | 1,391.3 |
| none + coregraphics | latn | 32px | 1,339.3 |
| none + coregraphics | latn | 64px | 1,308.8 |
| none + coregraphics | latn | 128px | 1,112.7 |
| none + coregraphics | mixd | 16px | 6,122.9 |
| none + coregraphics | mixd | 32px | 6,390.8 |
| none + coregraphics | mixd | 64px | 5,922.6 |
| none + coregraphics | mixd | 128px | 3,056.6 |
| none + orge | arab | 16px | 5,371.7 |
| none + orge | arab | 32px | 2,866.6 |
| none + orge | arab | 64px | 704.0 |
| none + orge | arab | 128px | 338.7 |
| none + orge | latn | 16px | 3,870.0 |
| none + orge | latn | 32px | 1,876.5 |
| none + orge | latn | 64px | 716.5 |
| none + orge | latn | 128px | 230.1 |
| none + orge | mixd | 16px | 4,797.3 |
| none + orge | mixd | 32px | 2,053.3 |
| none + orge | mixd | 64px | 1,040.7 |
| none + orge | mixd | 128px | 343.9 |
| none + skia | arab | 16px | 3,077.3 |
| none + skia | arab | 32px | 1,926.9 |
| none + skia | arab | 64px | 961.6 |
| none + skia | arab | 128px | 191.8 |
| none + skia | latn | 16px | 2,247.8 |
| none + skia | latn | 32px | 1,333.1 |
| none + skia | latn | 64px | 587.6 |
| none + skia | latn | 128px | 255.2 |
| none + skia | mixd | 16px | 2,563.6 |
| none + skia | mixd | 32px | 1,879.1 |
| none + skia | mixd | 64px | 1,276.7 |
| none + skia | mixd | 128px | 551.4 |
| none + zeno | arab | 16px | 2,797.5 |
| none + zeno | arab | 32px | 2,438.5 |
| none + zeno | arab | 64px | 1,728.1 |
| none + zeno | arab | 128px | 976.4 |
| none + zeno | latn | 16px | 1,390.8 |
| none + zeno | latn | 32px | 718.9 |
| none + zeno | latn | 64px | 1,062.0 |
| none + zeno | latn | 128px | 717.8 |
| none + zeno | mixd | 16px | 4,456.5 |
| none + zeno | mixd | 32px | 3,726.5 |
| none + zeno | mixd | 64px | 2,709.0 |
| none + zeno | mixd | 128px | 1,240.0 |

---
*Community project by FontLab - https://www.fontlab.org/*
