<!-- this_file: test_reports/260211-224031/README.md -->
# Test Report

- **Generated:** 2026-02-11T21:41:18+00:00
- **Run started:** 2026-02-11T21:40:31+00:00
- **Run finished:** 2026-02-11T21:41:18+00:00
- **Quick mode:** `False`
- **AI analyses skipped:** `False`
- **Suite duration:** `45.558s`

## Overall

- **Total steps:** 18
- **Passed:** 18
- **Failed:** 0
- **Skipped:** 0
- **Required failures:** 0

## Category Summary

| Category | Passed | Failed | Skipped | Total |
|---|---:|---:|---:|---:|
| `ai_analysis` | 2 | 0 | 0 | 2 |
| `practical_tests` | 6 | 0 | 0 | 6 |
| `sanity_tests` | 3 | 0 | 0 | 3 |
| `smoke_tests` | 4 | 0 | 0 | 4 |
| `unit_tests` | 3 | 0 | 0 | 3 |

## Step Results

| Step ID | Category | Status | Duration | Required | Log |
|---|---|---|---:|---|---|
| `smoke_build_cli` | `smoke_tests` | PASS | 2.006s | yes | [log](logs/smoke_build_cli.log) |
| `smoke_version` | `smoke_tests` | PASS | 0.016s | yes | [log](logs/smoke_version.log) |
| `smoke_info` | `smoke_tests` | PASS | 0.006s | yes | [log](logs/smoke_info.log) |
| `smoke_render_help` | `smoke_tests` | PASS | 0.006s | yes | [log](logs/smoke_render_help.log) |
| `sanity_fmt` | `sanity_tests` | PASS | 0.266s | yes | [log](logs/sanity_fmt.log) |
| `sanity_clippy` | `sanity_tests` | PASS | 3.231s | yes | [log](logs/sanity_clippy.log) |
| `sanity_list_tests` | `sanity_tests` | PASS | 7.966s | yes | [log](logs/sanity_list_tests.log) |
| `unit_lib_tests` | `unit_tests` | PASS | 0.523s | yes | [log](logs/unit_lib_tests.log) |
| `unit_integration_tests` | `unit_tests` | PASS | 1.172s | yes | [log](logs/unit_integration_tests.log) |
| `unit_doc_tests` | `unit_tests` | PASS | 3.38s | yes | [log](logs/unit_doc_tests.log) |
| `practical_render_latin_png` | `practical_tests` | PASS | 0.01s | yes | [log](logs/practical_render_latin_png.log) |
| `practical_render_arabic_png` | `practical_tests` | PASS | 0.009s | yes | [log](logs/practical_render_arabic_png.log) |
| `practical_render_variable_png` | `practical_tests` | PASS | 0.01s | yes | [log](logs/practical_render_variable_png.log) |
| `practical_render_mixed_svg` | `practical_tests` | PASS | 0.008s | yes | [log](logs/practical_render_mixed_svg.log) |
| `practical_batch_jobs` | `practical_tests` | PASS | 0.01s | yes | [log](logs/practical_batch_jobs.log) |
| `practical_artifact_validation` | `practical_tests` | PASS | 0.001s | yes | [log](logs/practical_artifact_validation.log) |
| `ai_codex_analysis` | `ai_analysis` | PASS | 5.839s | no | [log](logs/ai_codex_analysis.log) |
| `ai_gemini_analysis` | `ai_analysis` | PASS | 21.099s | no | [log](logs/ai_gemini_analysis.log) |

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
