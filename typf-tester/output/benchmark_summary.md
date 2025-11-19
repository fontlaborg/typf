# TYPF Benchmark Summary

**Date**: 2025-11-19 19:20:40  
**Iterations**: 100  
**Success Rate**: 240/240

## ⚠️ Performance Regressions Detected

**35 backend(s)** are >10% slower than baseline:

| Backend | Text | Size | Baseline | Current | Slowdown |

|---------|------|------|----------|---------|----------|
| none + JSON | latn | 16.0px | 0.047ms | 0.101ms | +116.1% |
| none + JSON | latn | 32.0px | 0.045ms | 0.155ms | +245.1% |
| none + JSON | latn | 64.0px | 0.043ms | 0.171ms | +296.6% |
| none + JSON | latn | 128.0px | 0.045ms | 0.126ms | +179.9% |
| none + JSON | arab | 16.0px | 0.064ms | 0.224ms | +251.2% |
| none + JSON | arab | 32.0px | 0.054ms | 0.101ms | +86.1% |
| none + JSON | arab | 64.0px | 0.051ms | 0.198ms | +291.2% |
| none + JSON | arab | 128.0px | 0.050ms | 0.066ms | +31.5% |
| none + JSON | mixd | 16.0px | 0.110ms | 0.249ms | +127.2% |
| none + JSON | mixd | 32.0px | 0.090ms | 0.144ms | +60.5% |

*...and 25 more (see benchmark_report.json)*


## Detailed Performance (Ops/sec)

| Backend | Text | Size | Ops/sec |

|:---|:---|:---:|---:|
| HarfBuzz + JSON | arab | 16px | 17,685.2 |
| HarfBuzz + JSON | arab | 32px | 18,893.5 |
| HarfBuzz + JSON | arab | 64px | 17,946.7 |
| HarfBuzz + JSON | arab | 128px | 17,721.9 |
| HarfBuzz + JSON | latn | 16px | 23,370.4 |
| HarfBuzz + JSON | latn | 32px | 22,020.0 |
| HarfBuzz + JSON | latn | 64px | 22,412.1 |
| HarfBuzz + JSON | latn | 128px | 22,464.5 |
| HarfBuzz + JSON | mixd | 16px | 9,403.0 |
| HarfBuzz + JSON | mixd | 32px | 10,672.3 |
| HarfBuzz + JSON | mixd | 64px | 10,194.8 |
| HarfBuzz + JSON | mixd | 128px | 10,350.3 |
| HarfBuzz + coregraphics | arab | 16px | 3,176.3 |
| HarfBuzz + coregraphics | arab | 32px | 5,198.5 |
| HarfBuzz + coregraphics | arab | 64px | 4,438.5 |
| HarfBuzz + coregraphics | arab | 128px | 2,990.0 |
| HarfBuzz + coregraphics | latn | 16px | 1,307.4 |
| HarfBuzz + coregraphics | latn | 32px | 1,312.0 |
| HarfBuzz + coregraphics | latn | 64px | 1,202.7 |
| HarfBuzz + coregraphics | latn | 128px | 979.9 |
| HarfBuzz + coregraphics | mixd | 16px | 4,909.7 |
| HarfBuzz + coregraphics | mixd | 32px | 5,315.8 |
| HarfBuzz + coregraphics | mixd | 64px | 5,618.0 |
| HarfBuzz + coregraphics | mixd | 128px | 4,080.5 |
| HarfBuzz + orge | arab | 16px | 2,474.9 |
| HarfBuzz + orge | arab | 32px | 1,495.7 |
| HarfBuzz + orge | arab | 64px | 682.8 |
| HarfBuzz + orge | arab | 128px | 349.9 |
| HarfBuzz + orge | latn | 16px | 3,358.4 |
| HarfBuzz + orge | latn | 32px | 1,735.7 |
| HarfBuzz + orge | latn | 64px | 705.8 |
| HarfBuzz + orge | latn | 128px | 206.4 |
| HarfBuzz + orge | mixd | 16px | 4,078.4 |
| HarfBuzz + orge | mixd | 32px | 2,568.2 |
| HarfBuzz + orge | mixd | 64px | 1,164.6 |
| HarfBuzz + orge | mixd | 128px | 386.3 |
| HarfBuzz + skia | arab | 16px | 3,286.0 |
| HarfBuzz + skia | arab | 32px | 1,927.0 |
| HarfBuzz + skia | arab | 64px | 1,145.9 |
| HarfBuzz + skia | arab | 128px | 512.3 |
| HarfBuzz + skia | latn | 16px | 2,510.1 |
| HarfBuzz + skia | latn | 32px | 1,116.0 |
| HarfBuzz + skia | latn | 64px | 91.1 |
| HarfBuzz + skia | latn | 128px | 117.9 |
| HarfBuzz + skia | mixd | 16px | 3,119.5 |
| HarfBuzz + skia | mixd | 32px | 2,049.6 |
| HarfBuzz + skia | mixd | 64px | 1,216.1 |
| HarfBuzz + skia | mixd | 128px | 539.2 |
| HarfBuzz + zeno | arab | 16px | 2,709.8 |
| HarfBuzz + zeno | arab | 32px | 2,283.1 |
| HarfBuzz + zeno | arab | 64px | 1,811.9 |
| HarfBuzz + zeno | arab | 128px | 993.4 |
| HarfBuzz + zeno | latn | 16px | 1,851.3 |
| HarfBuzz + zeno | latn | 32px | 1,633.3 |
| HarfBuzz + zeno | latn | 64px | 1,234.3 |
| HarfBuzz + zeno | latn | 128px | 685.8 |
| HarfBuzz + zeno | mixd | 16px | 4,185.0 |
| HarfBuzz + zeno | mixd | 32px | 3,557.7 |
| HarfBuzz + zeno | mixd | 64px | 2,415.3 |
| HarfBuzz + zeno | mixd | 128px | 1,145.7 |
| ICU-HarfBuzz + JSON | arab | 16px | 15,226.3 |
| ICU-HarfBuzz + JSON | arab | 32px | 16,486.0 |
| ICU-HarfBuzz + JSON | arab | 64px | 15,130.4 |
| ICU-HarfBuzz + JSON | arab | 128px | 16,138.1 |
| ICU-HarfBuzz + JSON | latn | 16px | 20,357.6 |
| ICU-HarfBuzz + JSON | latn | 32px | 19,409.0 |
| ICU-HarfBuzz + JSON | latn | 64px | 19,964.6 |
| ICU-HarfBuzz + JSON | latn | 128px | 19,814.4 |
| ICU-HarfBuzz + JSON | mixd | 16px | 8,603.6 |
| ICU-HarfBuzz + JSON | mixd | 32px | 8,885.2 |
| ICU-HarfBuzz + JSON | mixd | 64px | 7,034.9 |
| ICU-HarfBuzz + JSON | mixd | 128px | 8,406.4 |
| ICU-HarfBuzz + coregraphics | arab | 16px | 3,079.4 |
| ICU-HarfBuzz + coregraphics | arab | 32px | 5,069.1 |
| ICU-HarfBuzz + coregraphics | arab | 64px | 3,980.0 |
| ICU-HarfBuzz + coregraphics | arab | 128px | 1,570.5 |
| ICU-HarfBuzz + coregraphics | latn | 16px | 1,389.4 |
| ICU-HarfBuzz + coregraphics | latn | 32px | 1,367.2 |
| ICU-HarfBuzz + coregraphics | latn | 64px | 1,327.4 |
| ICU-HarfBuzz + coregraphics | latn | 128px | 1,018.5 |
| ICU-HarfBuzz + coregraphics | mixd | 16px | 3,072.8 |
| ICU-HarfBuzz + coregraphics | mixd | 32px | 1,591.2 |
| ICU-HarfBuzz + coregraphics | mixd | 64px | 1,037.8 |
| ICU-HarfBuzz + coregraphics | mixd | 128px | 3,158.1 |
| ICU-HarfBuzz + orge | arab | 16px | 5,196.8 |
| ICU-HarfBuzz + orge | arab | 32px | 2,918.0 |
| ICU-HarfBuzz + orge | arab | 64px | 1,128.3 |
| ICU-HarfBuzz + orge | arab | 128px | 405.0 |
| ICU-HarfBuzz + orge | latn | 16px | 3,278.2 |
| ICU-HarfBuzz + orge | latn | 32px | 1,635.7 |
| ICU-HarfBuzz + orge | latn | 64px | 613.6 |
| ICU-HarfBuzz + orge | latn | 128px | 200.5 |
| ICU-HarfBuzz + orge | mixd | 16px | 4,466.3 |
| ICU-HarfBuzz + orge | mixd | 32px | 2,786.9 |
| ICU-HarfBuzz + orge | mixd | 64px | 1,289.5 |
| ICU-HarfBuzz + orge | mixd | 128px | 442.7 |
| ICU-HarfBuzz + skia | arab | 16px | 3,181.0 |
| ICU-HarfBuzz + skia | arab | 32px | 2,136.5 |
| ICU-HarfBuzz + skia | arab | 64px | 1,086.9 |
| ICU-HarfBuzz + skia | arab | 128px | 479.1 |
| ICU-HarfBuzz + skia | latn | 16px | 2,292.0 |
| ICU-HarfBuzz + skia | latn | 32px | 1,276.5 |
| ICU-HarfBuzz + skia | latn | 64px | 566.1 |
| ICU-HarfBuzz + skia | latn | 128px | 235.7 |
| ICU-HarfBuzz + skia | mixd | 16px | 2,979.4 |
| ICU-HarfBuzz + skia | mixd | 32px | 1,742.6 |
| ICU-HarfBuzz + skia | mixd | 64px | 1,084.9 |
| ICU-HarfBuzz + skia | mixd | 128px | 497.6 |
| ICU-HarfBuzz + zeno | arab | 16px | 2,499.8 |
| ICU-HarfBuzz + zeno | arab | 32px | 2,368.3 |
| ICU-HarfBuzz + zeno | arab | 64px | 1,583.4 |
| ICU-HarfBuzz + zeno | arab | 128px | 894.3 |
| ICU-HarfBuzz + zeno | latn | 16px | 1,759.4 |
| ICU-HarfBuzz + zeno | latn | 32px | 1,639.3 |
| ICU-HarfBuzz + zeno | latn | 64px | 1,158.1 |
| ICU-HarfBuzz + zeno | latn | 128px | 694.5 |
| ICU-HarfBuzz + zeno | mixd | 16px | 3,568.6 |
| ICU-HarfBuzz + zeno | mixd | 32px | 3,296.2 |
| ICU-HarfBuzz + zeno | mixd | 64px | 2,125.4 |
| ICU-HarfBuzz + zeno | mixd | 128px | 1,090.0 |
| coretext + JSON | arab | 16px | 24,640.9 |
| coretext + JSON | arab | 32px | 18,294.6 |
| coretext + JSON | arab | 64px | 22,055.4 |
| coretext + JSON | arab | 128px | 23,451.0 |
| coretext + JSON | latn | 16px | 27,708.5 |
| coretext + JSON | latn | 32px | 26,370.1 |
| coretext + JSON | latn | 64px | 28,734.3 |
| coretext + JSON | latn | 128px | 27,024.3 |
| coretext + JSON | mixd | 16px | 11,312.4 |
| coretext + JSON | mixd | 32px | 9,075.7 |
| coretext + JSON | mixd | 64px | 9,127.8 |
| coretext + JSON | mixd | 128px | 11,909.0 |
| coretext + coregraphics | arab | 16px | 3,557.2 |
| coretext + coregraphics | arab | 32px | 6,712.2 |
| coretext + coregraphics | arab | 64px | 5,206.0 |
| coretext + coregraphics | arab | 128px | 3,543.4 |
| coretext + coregraphics | latn | 16px | 1,337.5 |
| coretext + coregraphics | latn | 32px | 1,332.9 |
| coretext + coregraphics | latn | 64px | 1,266.3 |
| coretext + coregraphics | latn | 128px | 1,043.0 |
| coretext + coregraphics | mixd | 16px | 4,851.0 |
| coretext + coregraphics | mixd | 32px | 5,408.2 |
| coretext + coregraphics | mixd | 64px | 4,590.9 |
| coretext + coregraphics | mixd | 128px | 3,424.7 |
| coretext + orge | arab | 16px | 5,836.3 |
| coretext + orge | arab | 32px | 3,152.0 |
| coretext + orge | arab | 64px | 1,323.4 |
| coretext + orge | arab | 128px | 461.8 |
| coretext + orge | latn | 16px | 3,669.8 |
| coretext + orge | latn | 32px | 1,720.1 |
| coretext + orge | latn | 64px | 654.6 |
| coretext + orge | latn | 128px | 204.6 |
| coretext + orge | mixd | 16px | 4,368.3 |
| coretext + orge | mixd | 32px | 2,592.1 |
| coretext + orge | mixd | 64px | 1,175.7 |
| coretext + orge | mixd | 128px | 425.8 |
| coretext + skia | arab | 16px | 3,975.9 |
| coretext + skia | arab | 32px | 2,401.5 |
| coretext + skia | arab | 64px | 1,301.4 |
| coretext + skia | arab | 128px | 573.7 |
| coretext + skia | latn | 16px | 2,546.1 |
| coretext + skia | latn | 32px | 1,295.1 |
| coretext + skia | latn | 64px | 563.6 |
| coretext + skia | latn | 128px | 239.7 |
| coretext + skia | mixd | 16px | 3,545.2 |
| coretext + skia | mixd | 32px | 2,279.3 |
| coretext + skia | mixd | 64px | 1,218.1 |
| coretext + skia | mixd | 128px | 512.8 |
| coretext + zeno | arab | 16px | 3,170.2 |
| coretext + zeno | arab | 32px | 2,722.4 |
| coretext + zeno | arab | 64px | 2,105.1 |
| coretext + zeno | arab | 128px | 1,197.3 |
| coretext + zeno | latn | 16px | 1,940.9 |
| coretext + zeno | latn | 32px | 1,812.3 |
| coretext + zeno | latn | 64px | 1,398.2 |
| coretext + zeno | latn | 128px | 763.9 |
| coretext + zeno | mixd | 16px | 3,845.0 |
| coretext + zeno | mixd | 32px | 3,224.3 |
| coretext + zeno | mixd | 64px | 2,379.3 |
| coretext + zeno | mixd | 128px | 1,181.7 |
| none + JSON | arab | 16px | 4,467.5 |
| none + JSON | arab | 32px | 9,931.4 |
| none + JSON | arab | 64px | 5,043.4 |
| none + JSON | arab | 128px | 15,116.6 |
| none + JSON | latn | 16px | 9,891.4 |
| none + JSON | latn | 32px | 6,453.0 |
| none + JSON | latn | 64px | 5,848.9 |
| none + JSON | latn | 128px | 7,906.6 |
| none + JSON | mixd | 16px | 4,008.8 |
| none + JSON | mixd | 32px | 6,928.9 |
| none + JSON | mixd | 64px | 8,894.2 |
| none + JSON | mixd | 128px | 6,483.1 |
| none + coregraphics | arab | 16px | 4,479.0 |
| none + coregraphics | arab | 32px | 7,866.1 |
| none + coregraphics | arab | 64px | 5,802.3 |
| none + coregraphics | arab | 128px | 3,520.0 |
| none + coregraphics | latn | 16px | 1,267.2 |
| none + coregraphics | latn | 32px | 1,425.8 |
| none + coregraphics | latn | 64px | 1,345.6 |
| none + coregraphics | latn | 128px | 1,160.2 |
| none + coregraphics | mixd | 16px | 7,120.2 |
| none + coregraphics | mixd | 32px | 7,483.5 |
| none + coregraphics | mixd | 64px | 6,981.0 |
| none + coregraphics | mixd | 128px | 4,675.7 |
| none + orge | arab | 16px | 4,055.5 |
| none + orge | arab | 32px | 2,525.8 |
| none + orge | arab | 64px | 903.6 |
| none + orge | arab | 128px | 308.0 |
| none + orge | latn | 16px | 3,370.5 |
| none + orge | latn | 32px | 1,576.2 |
| none + orge | latn | 64px | 646.3 |
| none + orge | latn | 128px | 213.9 |
| none + orge | mixd | 16px | 3,131.6 |
| none + orge | mixd | 32px | 2,139.0 |
| none + orge | mixd | 64px | 1,121.6 |
| none + orge | mixd | 128px | 427.3 |
| none + skia | arab | 16px | 3,515.0 |
| none + skia | arab | 32px | 2,010.6 |
| none + skia | arab | 64px | 1,067.9 |
| none + skia | arab | 128px | 429.8 |
| none + skia | latn | 16px | 2,678.9 |
| none + skia | latn | 32px | 1,358.4 |
| none + skia | latn | 64px | 618.8 |
| none + skia | latn | 128px | 260.3 |
| none + skia | mixd | 16px | 3,947.5 |
| none + skia | mixd | 32px | 2,351.0 |
| none + skia | mixd | 64px | 1,431.2 |
| none + skia | mixd | 128px | 599.8 |
| none + zeno | arab | 16px | 2,913.8 |
| none + zeno | arab | 32px | 2,234.5 |
| none + zeno | arab | 64px | 1,780.1 |
| none + zeno | arab | 128px | 973.8 |
| none + zeno | latn | 16px | 2,064.8 |
| none + zeno | latn | 32px | 1,761.6 |
| none + zeno | latn | 64px | 1,367.3 |
| none + zeno | latn | 128px | 762.8 |
| none + zeno | mixd | 16px | 5,199.0 |
| none + zeno | mixd | 32px | 4,627.1 |
| none + zeno | mixd | 64px | 2,960.4 |
| none + zeno | mixd | 128px | 1,314.0 |

---
*Made by FontLab - https://www.fontlab.com/*
