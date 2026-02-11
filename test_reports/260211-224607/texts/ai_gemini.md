Here's a breakdown of the test suite summary:

**Top Risks:**
1.  **Codebase Instability:** Multiple critical checks (`build`, `lint`, `format`, `unit tests`) are failing.
2.  **Maintenance Barrier:** Failing tests make it dangerous to introduce changes, increasing technical debt.
3.  **Inconsistent Quality:** Failures in formatting and linting suggest a lack of adherence to coding standards.
4.  **Test Execution Issues:** `sanity_list_tests` failing is critical, indicating potential problems with how tests are discovered or run.

**Probable Root Causes:**
1.  **Recent Code Changes:** A large influx of new code or refactoring may have broken existing tests.
2.  **Environment Misconfiguration:** The development or CI environment might not be set up correctly for all tools (e.g., Rust toolchain, linters).
3.  **Test Degradation:** Tests may be outdated, flaky, or not properly maintained.
4.  **Incomplete Orchestration:** The main test script (`test.sh` or `test.py`) might not be correctly executing all required steps or handling their outputs.

**Concrete Next Actions:**
1.  **Execute `test.sh`:** This script likely orchestrates all the mentioned tests. Its output will be crucial for identifying specific failure points.
2.  **Investigate Build Failures:** Address `smoke_build_cli` first, as a non-building project is fundamental.
3.  **Address Formatting/Linting:** Fix `sanity_fmt` and `sanity_clippy` for immediate code quality improvement.
4.  **Diagnose Test Failures:** For `unit_lib_tests`, `unit_integration_tests`, `unit_doc_tests`, and `sanity_list_tests`, examine the output of `test.sh` to pinpoint the exact failing tests and their error messages.
5.  **Fix Failing Tests:** Based on error messages, debug and correct the code or the tests themselves.

Let's start by running the primary test script.
The `run_shell_command` tool is not available. I need to find an alternative way to execute the `test.sh` script.

Since I cannot directly execute shell commands, I will try to use `codebase_investigator` to understand the `test.sh` script's content and then attempt to simulate its execution if possible, or at least understand what commands it runs. If that fails, I will try to use `read_file` to examine the script's content and manually infer the commands.

Let's start by examining `test.sh`.
The `test.sh` script is a simple wrapper that executes another script at `./scripts/test.sh`. I need to read the content of `./scripts/test.sh` to understand what tests it runs.
