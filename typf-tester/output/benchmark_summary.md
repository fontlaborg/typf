# TYPF Benchmark Summary

**Date**: 2025-11-19 03:42:44  
**Iterations**: 100  
**Success Rate**: 80/80

## Backend Performance

| Backend | Avg Time (ms) | Ops/sec | Success |

|---------|---------------|---------|---------|
| HarfBuzz + JSON | 0.043 | 23188 | 100% |
| HarfBuzz + coregraphics | 0.062 | 17222 | 100% |
| HarfBuzz + orge | 1.701 | 1454 | 100% |
| HarfBuzz + skia | 0.227 | 5441 | 100% |
| HarfBuzz + zeno | 0.394 | 2539 | 100% |
| ICU-HarfBuzz + JSON | 0.045 | 22110 | 100% |
| ICU-HarfBuzz + coregraphics | 0.063 | 17029 | 100% |
| ICU-HarfBuzz + orge | 1.690 | 1450 | 100% |
| ICU-HarfBuzz + skia | 0.228 | 5450 | 100% |
| ICU-HarfBuzz + zeno | 0.399 | 2507 | 100% |
| coretext + JSON | 0.034 | 29355 | 100% |
| coretext + coregraphics | 0.052 | 20780 | 100% |
| coretext + orge | 1.607 | 1496 | 100% |
| coretext + skia | 0.215 | 5830 | 100% |
| coretext + zeno | 0.384 | 2607 | 100% |
| none + JSON | 0.041 | 24335 | 100% |
| none + coregraphics | 0.062 | 18103 | 100% |
| none + orge | 1.828 | 1469 | 100% |
| none + skia | 0.227 | 5472 | 100% |
| none + zeno | 0.402 | 2488 | 100% |

## Performance by Text Type

| Text | Avg Time (ms) | Ops/sec |

|------|---------------|---------|
| latn | 0.485 | 10516 |

---
*Made by FontLab - https://www.fontlab.com/*
