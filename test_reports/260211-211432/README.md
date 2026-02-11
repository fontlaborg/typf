<!-- this_file: test_reports/260211-211432/README.md -->
# Test Report

- **Generated:** 2026-02-11T20:15:19+00:00
- **Run started:** 2026-02-11T20:14:32+00:00
- **Run finished:** 2026-02-11T20:15:19+00:00
- **Quick mode:** `False`
- **AI analyses skipped:** `False`
- **Suite duration:** `45.058s`

## Overall

- **Total steps:** 18
- **Passed:** 16
- **Failed:** 2
- **Skipped:** 0
- **Required failures:** 2

## Category Summary

| Category | Passed | Failed | Skipped | Total |
|---|---:|---:|---:|---:|
| `ai_analysis` | 2 | 0 | 0 | 2 |
| `practical_tests` | 6 | 0 | 0 | 6 |
| `sanity_tests` | 1 | 2 | 0 | 3 |
| `smoke_tests` | 4 | 0 | 0 | 4 |
| `unit_tests` | 3 | 0 | 0 | 3 |

## Step Results

| Step ID | Category | Status | Duration | Required | Log |
|---|---|---|---:|---|---|
| `smoke_build_cli` | `smoke_tests` | PASS | 1.879s | yes | [log](logs/smoke_build_cli.log) |
| `smoke_version` | `smoke_tests` | PASS | 0.011s | yes | [log](logs/smoke_version.log) |
| `smoke_info` | `smoke_tests` | PASS | 0.005s | yes | [log](logs/smoke_info.log) |
| `smoke_render_help` | `smoke_tests` | PASS | 0.006s | yes | [log](logs/smoke_render_help.log) |
| `sanity_fmt` | `sanity_tests` | FAIL | 0.094s | yes | [log](logs/sanity_fmt.log) |
| `sanity_clippy` | `sanity_tests` | FAIL | 3.079s | yes | [log](logs/sanity_clippy.log) |
| `sanity_list_tests` | `sanity_tests` | PASS | 7.365s | yes | [log](logs/sanity_list_tests.log) |
| `unit_lib_tests` | `unit_tests` | PASS | 0.457s | yes | [log](logs/unit_lib_tests.log) |
| `unit_integration_tests` | `unit_tests` | PASS | 1.066s | yes | [log](logs/unit_integration_tests.log) |
| `unit_doc_tests` | `unit_tests` | PASS | 2.644s | yes | [log](logs/unit_doc_tests.log) |
| `practical_render_latin_png` | `practical_tests` | PASS | 0.01s | yes | [log](logs/practical_render_latin_png.log) |
| `practical_render_arabic_png` | `practical_tests` | PASS | 0.008s | yes | [log](logs/practical_render_arabic_png.log) |
| `practical_render_variable_png` | `practical_tests` | PASS | 0.01s | yes | [log](logs/practical_render_variable_png.log) |
| `practical_render_mixed_svg` | `practical_tests` | PASS | 0.007s | yes | [log](logs/practical_render_mixed_svg.log) |
| `practical_batch_jobs` | `practical_tests` | PASS | 0.009s | yes | [log](logs/practical_batch_jobs.log) |
| `practical_artifact_validation` | `practical_tests` | PASS | 0.0s | yes | [log](logs/practical_artifact_validation.log) |
| `ai_codex_analysis` | `ai_analysis` | PASS | 7.699s | no | [log](logs/ai_codex_analysis.log) |
| `ai_gemini_analysis` | `ai_analysis` | PASS | 20.709s | no | [log](logs/ai_gemini_analysis.log) |

## Core Files

- [Summary JSON](json/summary.json)
- [Results JSON](json/results.json)
- [Environment JSON](json/environment.json)
- [Practical Checks JSON](json/practical_checks.json)
- [Metrics JSON](json/metrics.json)
- [Command List](texts/commands.txt)
- [Summary Text](texts/summary.txt)
- [Practical Checks Text](texts/practical_checks.txt)
- [AI Prompt](texts/ai_prompt.txt)
- [Codex Analysis](texts/ai_codex.md)
- [Gemini Analysis](texts/ai_gemini.md)

## Logs

- [Logs Directory](logs/)

## Images

- [arabic.png](images/arabic.png)
- [latin.png](images/latin.png)
- [variable_font.png](images/variable_font.png)

## Other Artifacts

- [artifacts/batch_output/batch_arabic.svg](artifacts/batch_output/batch_arabic.svg)
- [artifacts/batch_output/batch_latin.png](artifacts/batch_output/batch_latin.png)
- [artifacts/mixed_scripts.svg](artifacts/mixed_scripts.svg)
