**Top Risks:**
*   Codebase style fragmentation and inconsistency.
*   Potential introduction of bugs or code smells due to unmet linting rules.
*   Reduced maintainability and difficulty in code reviews.

**Probable Root Causes:**
*   `sanity_fmt` failure: Code does not conform to standard Rust formatting.
*   `sanity_clippy` failure: Code violates linting rules, possibly from new code or unaddressed warnings.

**Concrete Next Actions:**
1.  Format the codebase using `cargo fmt`.
2.  Attempt to fix linting issues with `cargo clippy --fix`.
3.  Re-run tests to confirm resolution.

The `run_shell_command` tool is not available. I need to find an alternative way to execute these commands or inform the user about this limitation. Given the available tools, I cannot directly execute shell commands.

I will adapt my response to reflect this limitation. Since I cannot directly fix the formatting and linting issues, I will advise the user on how to do it.

Revised Plan:
1.  Identify top risks.
2.  Identify probable root causes.
3.  Provide instructions for the user on how to perform the next actions, as I cannot execute them directly.

**Top Risks:**
*   Codebase style fragmentation and inconsistency.
*   Potential introduction of bugs or code smells due to unmet linting rules.
*   Reduced maintainability and difficulty in code reviews.

**Probable Root Causes:**
*   `sanity_fmt` failure: Code does not conform to standard Rust formatting.
*   `sanity_clippy` failure: Code violates linting rules, possibly from new code or unaddressed warnings.

**Concrete Next Actions:**
1.  **Format Code:** Run `cargo fmt` in your project's root directory to automatically format the code.
2.  **Fix Linting Issues:** Run `cargo clippy --fix` to attempt to automatically fix linting issues.
3.  **Re-run Tests:** After applying the above, re-run the test suite to verify that the `sanity_fmt` and `sanity_clippy` checks now pass.
4.  **Manual Review:** If any issues persist after running the commands, manually address them based on the output of `cargo fmt --check` and `cargo clippy`.**Top Risks:**
*   Codebase style fragmentation and inconsistency.
*   Potential introduction of bugs or code smells due to unmet linting rules.
*   Reduced maintainability and difficulty in code reviews.

**Probable Root Causes:**
*   `sanity_fmt` failure: Code does not conform to standard Rust formatting.
*   `sanity_clippy` failure: Code violates linting rules, possibly from new code or unaddressed warnings.

**Concrete Next Actions:**
1.  **Format Code:** Run `cargo fmt` in your project's root directory to automatically format the code.
2.  **Fix Linting Issues:** Run `cargo clippy --fix` to attempt to automatically fix linting issues.
3.  **Re-run Tests:** After applying the above, re-run the test suite to verify that the `sanity_fmt` and `sanity_clippy` checks now pass.
4.  **Manual Review:** If any issues persist after running the commands, manually address them based on the output of `cargo fmt --check` and `cargo clippy`.
