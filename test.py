#!/usr/bin/env python3
# this_file: test.py
"""Comprehensive automated test runner with timestamped report artifacts.

Usage:
  python3 test.py
  python3 test.py --quick
  python3 test.py --skip-ai
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import struct
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional


ROOT = Path(__file__).resolve().parent
REPORTS_ROOT = ROOT / "test_reports"


@dataclass
class Step:
    """Single command-driven test step."""

    step_id: str
    category: str
    title: str
    description: str
    command: List[str]
    timeout_seconds: int = 1800
    required: bool = True
    stdin_text: Optional[str] = None


@dataclass
class StepResult:
    """Execution result for one test step."""

    step_id: str
    category: str
    title: str
    status: str
    required: bool
    return_code: Optional[int]
    duration_seconds: float
    started_at: str
    finished_at: str
    command: List[str]
    log_file: str
    artifacts: List[str]
    notes: str


def iso_now() -> str:
    """UTC timestamp with seconds precision."""
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def quote_command(command: List[str]) -> str:
    """Printable shell-ish representation of a command."""
    return " ".join(subprocess.list2cmdline([part]) for part in command)


def tool_exists(name: str) -> bool:
    """Return True if executable is on PATH."""
    return shutil.which(name) is not None


def run_capture(command: List[str], cwd: Path, timeout: int = 30) -> str:
    """Run a small command and return combined output."""
    try:
        proc = subprocess.run(
            command,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except Exception as exc:  # pragma: no cover - defensive
        return f"<error: {exc}>"
    output = (proc.stdout or "") + (proc.stderr or "")
    return output.strip()


def write_json(path: Path, payload: Dict) -> None:
    """Write JSON with stable formatting."""
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


class TestRunner:
    """Runs the complete suite and generates report artifacts."""

    def __init__(self, quick: bool, skip_ai: bool, fail_fast: bool) -> None:
        self.quick = quick
        self.skip_ai = skip_ai
        self.fail_fast = fail_fast
        self.timestamp = datetime.now().strftime("%y%m%d-%H%M%S")
        self.report_dir = REPORTS_ROOT / self.timestamp
        self.logs_dir = self.report_dir / "logs"
        self.json_dir = self.report_dir / "json"
        self.text_dir = self.report_dir / "texts"
        self.images_dir = self.report_dir / "images"
        self.artifacts_dir = self.report_dir / "artifacts"
        self.results: List[StepResult] = []
        self.practical_checks: List[Dict[str, object]] = []
        self.expected_practical_outputs: Dict[str, Path] = {}
        self.run_started_at = iso_now()

    def setup_dirs(self) -> None:
        """Create report directories."""
        for directory in [
            self.report_dir,
            self.logs_dir,
            self.json_dir,
            self.text_dir,
            self.images_dir,
            self.artifacts_dir,
        ]:
            directory.mkdir(parents=True, exist_ok=True)

    def build_steps(self) -> List[Step]:
        """Define the test suite."""
        jobs_file = self.text_dir / "batch_jobs.jsonl"
        jobs_file.write_text(
            "\n".join(
                [
                    json.dumps(
                        {
                            "text": "Batch Latin Smoke",
                            "font": "test-fonts/NotoSans-Regular.ttf",
                            "output": "batch_latin.png",
                            "size": 44,
                            "format": "png",
                            "shaper": "hb",
                            "renderer": "opixa",
                        }
                    ),
                    json.dumps(
                        {
                            "text": "اختبار دفعة عربية",
                            "font": "test-fonts/NotoNaskhArabic-Regular.ttf",
                            "output": "batch_arabic.svg",
                            "size": 54,
                            "format": "svg",
                            "shaper": "hb",
                            "renderer": "opixa",
                            "language": "ar",
                        }
                    ),
                ]
            )
            + "\n",
            encoding="utf-8",
        )

        self.expected_practical_outputs = {
            "latin_png": self.images_dir / "latin.png",
            "arabic_png": self.images_dir / "arabic.png",
            "variable_png": self.images_dir / "variable_font.png",
            "mixed_svg": self.artifacts_dir / "mixed_scripts.svg",
            "batch_latin_png": self.artifacts_dir / "batch_output" / "batch_latin.png",
            "batch_arabic_svg": self.artifacts_dir / "batch_output" / "batch_arabic.svg",
        }

        typf_bin = ["target/debug/typf"]
        steps: List[Step] = [
            Step(
                step_id="smoke_build_cli",
                category="smoke_tests",
                title="Build CLI",
                description="Build typf CLI in debug mode.",
                command=["cargo", "build", "-p", "typf-cli"],
                timeout_seconds=2400,
            ),
            Step(
                step_id="smoke_version",
                category="smoke_tests",
                title="CLI Version",
                description="Verify typf binary executes.",
                command=typf_bin + ["--version"],
            ),
            Step(
                step_id="smoke_info",
                category="smoke_tests",
                title="CLI Backend Info",
                description="Validate backend discovery command.",
                command=typf_bin + ["info"],
            ),
            Step(
                step_id="smoke_render_help",
                category="smoke_tests",
                title="CLI Render Help",
                description="Verify render command help path.",
                command=typf_bin + ["render", "--help"],
            ),
            Step(
                step_id="sanity_fmt",
                category="sanity_tests",
                title="Rust Format Check",
                description="Ensure formatting is stable.",
                command=["cargo", "fmt", "--all", "--check"],
                timeout_seconds=900,
            ),
            Step(
                step_id="sanity_clippy",
                category="sanity_tests",
                title="Rust Clippy",
                description="Run strict lint checks.",
                command=["cargo", "clippy", "--workspace", "--all-features", "--all-targets", "--", "-D", "warnings"],
                timeout_seconds=2400,
                required=not self.quick,
            ),
            Step(
                step_id="sanity_list_tests",
                category="sanity_tests",
                title="List Rust Tests",
                description="Inventory all workspace tests.",
                command=["cargo", "test", "--workspace", "--all-features", "--", "--list"],
                timeout_seconds=2400,
            ),
            Step(
                step_id="unit_lib_tests",
                category="unit_tests",
                title="Rust Library Unit Tests",
                description="Run lib-focused tests.",
                command=["cargo", "test", "--workspace", "--all-features", "--lib"],
                timeout_seconds=3600,
            ),
            Step(
                step_id="unit_integration_tests",
                category="unit_tests",
                title="Rust Integration Tests",
                description="Run integration test targets.",
                command=["cargo", "test", "--workspace", "--all-features", "--tests"],
                timeout_seconds=3600,
                required=not self.quick,
            ),
            Step(
                step_id="unit_doc_tests",
                category="unit_tests",
                title="Rust Doc Tests",
                description="Run documentation examples as tests.",
                command=["cargo", "test", "--workspace", "--all-features", "--doc"],
                timeout_seconds=2400,
                required=not self.quick,
            ),
            Step(
                step_id="practical_render_latin_png",
                category="practical_tests",
                title="Render Latin PNG",
                description="Render Latin text to PNG.",
                command=typf_bin
                + [
                    "render",
                    "Hello, Typf Practical Test",
                    "-f",
                    "test-fonts/NotoSans-Regular.ttf",
                    "-s",
                    "60",
                    "--shaper",
                    "hb",
                    "--renderer",
                    "opixa",
                    "-O",
                    "png",
                    "-o",
                    str(self.expected_practical_outputs["latin_png"]),
                    "--quiet",
                ],
            ),
            Step(
                step_id="practical_render_arabic_png",
                category="practical_tests",
                title="Render Arabic PNG",
                description="Render Arabic RTL sample to PNG.",
                command=typf_bin
                + [
                    "render",
                    "مرحبا بالعالم",
                    "-f",
                    "test-fonts/NotoNaskhArabic-Regular.ttf",
                    "-s",
                    "64",
                    "--shaper",
                    "hb",
                    "--renderer",
                    "opixa",
                    "-O",
                    "png",
                    "-o",
                    str(self.expected_practical_outputs["arabic_png"]),
                    "--quiet",
                ],
            ),
            Step(
                step_id="practical_render_variable_png",
                category="practical_tests",
                title="Render Variable Font PNG",
                description="Render with variation coordinates.",
                command=typf_bin
                + [
                    "render",
                    "Variable Width",
                    "-f",
                    "test-fonts/Kalnia[wdth,wght].ttf",
                    "-i",
                    "wght=700,wdth=120",
                    "-s",
                    "70",
                    "--shaper",
                    "hb",
                    "--renderer",
                    "opixa",
                    "-O",
                    "png",
                    "-o",
                    str(self.expected_practical_outputs["variable_png"]),
                    "--quiet",
                ],
            ),
            Step(
                step_id="practical_render_mixed_svg",
                category="practical_tests",
                title="Render Mixed Script SVG",
                description="Render mixed script sample to SVG.",
                command=typf_bin
                + [
                    "render",
                    "Hello, مرحبا, 你好!",
                    "-f",
                    "test-fonts/NotoSans-Regular.ttf",
                    "-s",
                    "54",
                    "--shaper",
                    "hb",
                    "--renderer",
                    "opixa",
                    "-O",
                    "svg",
                    "-o",
                    str(self.expected_practical_outputs["mixed_svg"]),
                    "--quiet",
                ],
            ),
            Step(
                step_id="practical_batch_jobs",
                category="practical_tests",
                title="Batch Render Jobs",
                description="Run JSONL batch rendering path.",
                command=typf_bin
                + [
                    "batch",
                    "-i",
                    str(jobs_file),
                    "-o",
                    str(self.artifacts_dir / "batch_output"),
                    "--verbose",
                ],
                timeout_seconds=1200,
            ),
        ]
        return steps

    def run_step(self, step: Step) -> StepResult:
        """Execute one step and persist full logs."""
        started_wall = iso_now()
        started = time.monotonic()
        log_path = self.logs_dir / f"{step.step_id}.log"
        status = "pass"
        return_code: Optional[int] = None
        notes = ""

        if not step.command:
            status = "fail"
            notes = "Empty command."
            duration = time.monotonic() - started
            result = StepResult(
                step_id=step.step_id,
                category=step.category,
                title=step.title,
                status=status,
                required=step.required,
                return_code=None,
                duration_seconds=round(duration, 3),
                started_at=started_wall,
                finished_at=iso_now(),
                command=step.command,
                log_file=str(log_path.relative_to(self.report_dir)),
                artifacts=[],
                notes=notes,
            )
            log_path.write_text("Invalid step: empty command\n", encoding="utf-8")
            return result

        try:
            completed = subprocess.run(
                step.command,
                cwd=str(ROOT),
                input=step.stdin_text,
                capture_output=True,
                text=True,
                timeout=step.timeout_seconds,
                check=False,
            )
            return_code = completed.returncode
            if completed.returncode != 0:
                status = "fail"
                notes = f"Exit code {completed.returncode}"
            stdout = completed.stdout or ""
            stderr = completed.stderr or ""
        except FileNotFoundError as exc:
            status = "skip" if not step.required else "fail"
            notes = f"Command not found: {exc}"
            stdout, stderr = "", str(exc)
        except subprocess.TimeoutExpired as exc:
            status = "fail"
            return_code = None
            notes = f"Timeout after {step.timeout_seconds}s"
            stdout = exc.stdout or ""
            stderr = exc.stderr or ""
        except Exception as exc:  # pragma: no cover - defensive
            status = "fail"
            return_code = None
            notes = f"Unhandled error: {exc}"
            stdout = ""
            stderr = str(exc)

        duration = time.monotonic() - started
        finished_wall = iso_now()
        log_content = [
            f"step_id: {step.step_id}",
            f"title: {step.title}",
            f"category: {step.category}",
            f"required: {step.required}",
            f"started_at: {started_wall}",
            f"finished_at: {finished_wall}",
            f"duration_seconds: {duration:.3f}",
            f"status: {status}",
            f"return_code: {return_code}",
            f"command: {quote_command(step.command)}",
            "",
            "===== STDOUT =====",
            stdout,
            "",
            "===== STDERR =====",
            stderr,
            "",
        ]
        log_path.write_text("\n".join(log_content), encoding="utf-8")

        return StepResult(
            step_id=step.step_id,
            category=step.category,
            title=step.title,
            status=status,
            required=step.required,
            return_code=return_code,
            duration_seconds=round(duration, 3),
            started_at=started_wall,
            finished_at=finished_wall,
            command=step.command,
            log_file=str(log_path.relative_to(self.report_dir)),
            artifacts=[],
            notes=notes,
        )

    def check_png(self, path: Path) -> Dict[str, object]:
        """Validate PNG signature and dimensions."""
        if not path.exists():
            return {"ok": False, "reason": "missing"}
        data = path.read_bytes()
        if len(data) < 24:
            return {"ok": False, "reason": "too_small"}
        if data[:8] != b"\x89PNG\r\n\x1a\n":
            return {"ok": False, "reason": "bad_signature"}
        width, height = struct.unpack(">II", data[16:24])
        return {
            "ok": width > 0 and height > 0,
            "reason": "ok" if width > 0 and height > 0 else "invalid_dimensions",
            "bytes": len(data),
            "width": width,
            "height": height,
        }

    def check_svg(self, path: Path) -> Dict[str, object]:
        """Validate SVG structure."""
        if not path.exists():
            return {"ok": False, "reason": "missing"}
        text = path.read_text(encoding="utf-8", errors="replace")
        has_svg = "<svg" in text
        path_count = text.count("<path")
        return {
            "ok": has_svg and path_count > 0,
            "reason": "ok" if has_svg and path_count > 0 else "missing_svg_or_paths",
            "bytes": len(text.encode("utf-8")),
            "path_count": path_count,
        }

    def run_practical_checks(self) -> StepResult:
        """Validate generated practical artifacts."""
        started = time.monotonic()
        started_at = iso_now()
        checks: List[Dict[str, object]] = []
        png_keys = ["latin_png", "arabic_png", "variable_png", "batch_latin_png"]
        svg_keys = ["mixed_svg", "batch_arabic_svg"]
        for key in png_keys:
            path = self.expected_practical_outputs[key]
            payload = {"artifact": key, "path": str(path.relative_to(self.report_dir))}
            payload.update(self.check_png(path))
            checks.append(payload)
        for key in svg_keys:
            path = self.expected_practical_outputs[key]
            payload = {"artifact": key, "path": str(path.relative_to(self.report_dir))}
            payload.update(self.check_svg(path))
            checks.append(payload)

        self.practical_checks = checks
        practical_json = {"generated_at": iso_now(), "checks": checks}
        write_json(self.json_dir / "practical_checks.json", practical_json)

        lines = ["Practical Artifact Checks", "=======================", ""]
        for entry in checks:
            lines.append(
                f"- {entry['artifact']}: ok={entry['ok']} reason={entry['reason']} path={entry['path']}"
            )
        lines.append("")
        (self.text_dir / "practical_checks.txt").write_text("\n".join(lines), encoding="utf-8")
        failed = [entry for entry in checks if not bool(entry.get("ok"))]
        status = "pass" if not failed else "fail"
        notes = "All practical artifacts validated."
        if failed:
            names = ", ".join(str(entry["artifact"]) for entry in failed)
            notes = f"Artifact validation failed: {names}"

        log_path = self.logs_dir / "practical_artifact_validation.log"
        log_path.write_text("\n".join(lines), encoding="utf-8")

        return StepResult(
            step_id="practical_artifact_validation",
            category="practical_tests",
            title="Practical Artifact Validation",
            status=status,
            required=True,
            return_code=0 if status == "pass" else 1,
            duration_seconds=round(time.monotonic() - started, 3),
            started_at=started_at,
            finished_at=iso_now(),
            command=[],
            log_file=str(log_path.relative_to(self.report_dir)),
            artifacts=[
                str((self.json_dir / "practical_checks.json").relative_to(self.report_dir)),
                str((self.text_dir / "practical_checks.txt").relative_to(self.report_dir)),
            ],
            notes=notes,
        )

    def collect_environment(self) -> Dict[str, object]:
        """Capture runtime environment and tool versions."""
        env_payload = {
            "generated_at": iso_now(),
            "cwd": str(ROOT),
            "python": {
                "executable": sys.executable,
                "version": sys.version.replace("\n", " "),
            },
            "platform": {
                "system": platform.system(),
                "release": platform.release(),
                "machine": platform.machine(),
                "python_implementation": platform.python_implementation(),
            },
            "tools": {
                "cargo": run_capture(["cargo", "--version"], ROOT),
                "rustc": run_capture(["rustc", "--version"], ROOT),
                "uv": run_capture(["uv", "--version"], ROOT) if tool_exists("uv") else "<not_found>",
                "codex": run_capture(["codex", "--version"], ROOT) if tool_exists("codex") else "<not_found>",
                "gemini": run_capture(["gemini", "--version"], ROOT) if tool_exists("gemini") else "<not_found>",
            },
            "env_subset": {
                key: os.environ.get(key, "")
                for key in [
                    "CI",
                    "RUSTFLAGS",
                    "CARGO_TARGET_DIR",
                    "TYPF_CACHE",
                ]
            },
        }
        write_json(self.json_dir / "environment.json", env_payload)
        return env_payload

    def summarize(self) -> Dict[str, object]:
        """Create suite summary metrics."""
        by_category: Dict[str, Dict[str, int]] = {}
        totals = {"pass": 0, "fail": 0, "skip": 0}
        required_failures: List[str] = []
        for result in self.results:
            category = by_category.setdefault(
                result.category, {"pass": 0, "fail": 0, "skip": 0, "total": 0}
            )
            category["total"] += 1
            category[result.status] += 1
            totals[result.status] += 1
            if result.required and result.status != "pass":
                required_failures.append(result.step_id)

        suite_duration = round(
            sum(result.duration_seconds for result in self.results),
            3,
        )
        return {
            "generated_at": iso_now(),
            "run_started_at": self.run_started_at,
            "run_finished_at": iso_now(),
            "quick_mode": self.quick,
            "skip_ai": self.skip_ai,
            "counts": {
                "total_steps": len(self.results),
                **totals,
            },
            "categories": by_category,
            "required_failures": required_failures,
            "suite_duration_seconds": suite_duration,
        }

    def run_ai_analyses(self, pre_ai_summary: Dict[str, object]) -> None:
        """Run codex and gemini analyses and persist outputs."""
        summary_text = json.dumps(pre_ai_summary, indent=2, sort_keys=True)
        prompt = (
            "Review this test-suite summary and return:\n"
            "1) top risks\n"
            "2) probable root causes\n"
            "3) concrete next actions\n"
            "Keep it concise and actionable.\n\n"
            f"{summary_text}\n"
        )
        (self.text_dir / "ai_prompt.txt").write_text(prompt, encoding="utf-8")

        codex_output = self.text_dir / "ai_codex.md"
        gemini_output = self.text_dir / "ai_gemini.md"

        ai_steps: List[Step] = []
        if tool_exists("codex"):
            ai_steps.append(
                Step(
                    step_id="ai_codex_analysis",
                    category="ai_analysis",
                    title="Codex Analysis",
                    description="AI analysis over suite results.",
                    command=[
                        "codex",
                        "exec",
                        "--skip-git-repo-check",
                        "--sandbox",
                        "read-only",
                        "-C",
                        str(ROOT),
                        "-o",
                        str(codex_output),
                        "-",
                    ],
                    timeout_seconds=900,
                    required=False,
                    stdin_text=prompt,
                )
            )
        if tool_exists("gemini"):
            ai_steps.append(
                Step(
                    step_id="ai_gemini_analysis",
                    category="ai_analysis",
                    title="Gemini Analysis",
                    description="AI analysis over suite results.",
                    command=[
                        "gemini",
                        "--output-format",
                        "text",
                        "-p",
                        prompt,
                    ],
                    timeout_seconds=900,
                    required=False,
                )
            )

        if not ai_steps:
            self.results.append(
                StepResult(
                    step_id="ai_tools_unavailable",
                    category="ai_analysis",
                    title="AI Tools Availability",
                    status="fail",
                    required=True,
                    return_code=None,
                    duration_seconds=0.0,
                    started_at=iso_now(),
                    finished_at=iso_now(),
                    command=[],
                    log_file="logs/ai_tools_unavailable.log",
                    artifacts=[],
                    notes="Neither 'codex' nor 'gemini' is available on PATH.",
                )
            )
            (self.logs_dir / "ai_tools_unavailable.log").write_text(
                "Neither codex nor gemini was found on PATH.\n",
                encoding="utf-8",
            )
            codex_output.write_text("AI analysis unavailable: codex not found.\n", encoding="utf-8")
            gemini_output.write_text("AI analysis unavailable: gemini not found.\n", encoding="utf-8")
            return

        ai_successes = 0
        for step in ai_steps:
            result = self.run_step(step)
            self.results.append(result)
            if result.status == "pass":
                ai_successes += 1
            if step.step_id == "ai_gemini_analysis":
                gemini_log = self.report_dir / result.log_file
                if gemini_log.exists():
                    content = gemini_log.read_text(encoding="utf-8", errors="replace")
                    marker = "===== STDOUT ====="
                    if marker in content:
                        after = content.split(marker, 1)[1]
                        extracted = after.split("===== STDERR =====", 1)[0].strip()
                        gemini_output.write_text(extracted + "\n", encoding="utf-8")

        if ai_successes == 0:
            self.results.append(
                StepResult(
                    step_id="ai_minimum_requirement",
                    category="ai_analysis",
                    title="AI Analysis Requirement",
                    status="fail",
                    required=True,
                    return_code=None,
                    duration_seconds=0.0,
                    started_at=iso_now(),
                    finished_at=iso_now(),
                    command=[],
                    log_file="logs/ai_minimum_requirement.log",
                    artifacts=[],
                    notes="No AI analysis command succeeded.",
                )
            )
            (self.logs_dir / "ai_minimum_requirement.log").write_text(
                "No AI analysis command succeeded.\n",
                encoding="utf-8",
            )
            if not codex_output.exists():
                codex_output.write_text("Codex analysis failed.\n", encoding="utf-8")
            if not gemini_output.exists():
                gemini_output.write_text("Gemini analysis failed.\n", encoding="utf-8")

    def write_results_files(self, environment: Dict[str, object], summary: Dict[str, object]) -> None:
        """Write machine-readable files."""
        payload = {
            "generated_at": iso_now(),
            "environment": environment,
            "results": [asdict(result) for result in self.results],
        }
        write_json(self.json_dir / "results.json", payload)
        write_json(self.json_dir / "summary.json", summary)

        commands = [quote_command(result.command) for result in self.results if result.command]
        (self.text_dir / "commands.txt").write_text("\n".join(commands) + "\n", encoding="utf-8")
        summary_lines = [
            "Test Suite Summary",
            "==================",
            f"run_started_at: {summary['run_started_at']}",
            f"run_finished_at: {summary['run_finished_at']}",
            f"total_steps: {summary['counts']['total_steps']}",
            f"pass: {summary['counts']['pass']}",
            f"fail: {summary['counts']['fail']}",
            f"skip: {summary['counts']['skip']}",
            f"required_failures: {', '.join(summary['required_failures']) or 'none'}",
        ]
        (self.text_dir / "summary.txt").write_text("\n".join(summary_lines) + "\n", encoding="utf-8")

    def generate_readme(self, summary: Dict[str, object]) -> None:
        """Generate report index README linking all artifacts."""
        readme_path = self.report_dir / "README.md"
        relative_json_dir = "json"
        relative_logs_dir = "logs"
        relative_text_dir = "texts"
        relative_images_dir = "images"
        relative_artifacts_dir = "artifacts"

        category_rows = []
        for category, counts in sorted(summary["categories"].items()):
            category_rows.append(
                f"| `{category}` | {counts['pass']} | {counts['fail']} | {counts['skip']} | {counts['total']} |"
            )

        step_rows = []
        for result in self.results:
            step_rows.append(
                "| `{}` | `{}` | {} | {}s | {} | [{}]({}) |".format(
                    result.step_id,
                    result.category,
                    result.status.upper(),
                    result.duration_seconds,
                    "yes" if result.required else "no",
                    "log",
                    result.log_file.replace("\\", "/"),
                )
            )

        image_links = []
        for image in sorted(self.images_dir.glob("*")):
            image_links.append(f"- [{image.name}]({relative_images_dir}/{image.name})")

        artifact_links = []
        for artifact in sorted(self.artifacts_dir.rglob("*")):
            if artifact.is_file():
                rel = artifact.relative_to(self.report_dir).as_posix()
                artifact_links.append(f"- [{rel}]({rel})")

        lines = [
            f"<!-- this_file: test_reports/{self.timestamp}/README.md -->",
            "# Test Report",
            "",
            f"- **Generated:** {summary['generated_at']}",
            f"- **Run started:** {summary['run_started_at']}",
            f"- **Run finished:** {summary['run_finished_at']}",
            f"- **Quick mode:** `{summary['quick_mode']}`",
            f"- **AI analyses skipped:** `{summary['skip_ai']}`",
            f"- **Suite duration:** `{summary['suite_duration_seconds']}s`",
            "",
            "## Overall",
            "",
            f"- **Total steps:** {summary['counts']['total_steps']}",
            f"- **Passed:** {summary['counts']['pass']}",
            f"- **Failed:** {summary['counts']['fail']}",
            f"- **Skipped:** {summary['counts']['skip']}",
            f"- **Required failures:** {len(summary['required_failures'])}",
            "",
            "## Category Summary",
            "",
            "| Category | Passed | Failed | Skipped | Total |",
            "|---|---:|---:|---:|---:|",
        ]
        lines.extend(category_rows or ["| _none_ | 0 | 0 | 0 | 0 |"])

        lines.extend(
            [
                "",
                "## Step Results",
                "",
                "| Step ID | Category | Status | Duration | Required | Log |",
                "|---|---|---|---:|---|---|",
            ]
        )
        lines.extend(step_rows or ["| _none_ | _none_ | N/A | 0s | no | N/A |"])

        lines.extend(
            [
                "",
                "## Core Files",
                "",
                f"- [Summary JSON]({relative_json_dir}/summary.json)",
                f"- [Results JSON]({relative_json_dir}/results.json)",
                f"- [Environment JSON]({relative_json_dir}/environment.json)",
                f"- [Practical Checks JSON]({relative_json_dir}/practical_checks.json)",
                f"- [Metrics JSON]({relative_json_dir}/metrics.json)",
                f"- [Command List]({relative_text_dir}/commands.txt)",
                f"- [Summary Text]({relative_text_dir}/summary.txt)",
                f"- [Practical Checks Text]({relative_text_dir}/practical_checks.txt)",
                f"- [AI Prompt]({relative_text_dir}/ai_prompt.txt)",
                f"- [Codex Analysis]({relative_text_dir}/ai_codex.md)",
                f"- [Gemini Analysis]({relative_text_dir}/ai_gemini.md)",
                "",
                "## Logs",
                "",
                f"- [Logs Directory]({relative_logs_dir}/)",
                "",
                "## Images",
                "",
            ]
        )
        lines.extend(image_links or ["- _No images generated._"])

        lines.extend(["", "## Other Artifacts", ""])
        lines.extend(artifact_links or ["- _No additional artifacts generated._"])
        lines.append("")
        readme_path.write_text("\n".join(lines), encoding="utf-8")

    def create_latest_symlink(self) -> None:
        """Create/replace test_reports/latest symlink."""
        latest = REPORTS_ROOT / "latest"
        try:
            if latest.is_symlink() or latest.exists():
                latest.unlink()
            latest.symlink_to(self.report_dir.name)
        except OSError:
            # Symlink creation can fail on some environments; ignore.
            pass

    def extract_test_count_metric(self) -> None:
        """Parse test count from sanity_list_tests output into JSON."""
        metrics_path = self.json_dir / "metrics.json"
        list_step = next((r for r in self.results if r.step_id == "sanity_list_tests"), None)
        if not list_step:
            write_json(
                metrics_path,
                {
                    "generated_at": iso_now(),
                    "rust_tests_discovered": None,
                    "source_step": "sanity_list_tests",
                    "reason": "step_not_found",
                },
            )
            return
        log_path = self.report_dir / list_step.log_file
        if not log_path.exists():
            write_json(
                metrics_path,
                {
                    "generated_at": iso_now(),
                    "rust_tests_discovered": None,
                    "source_step": "sanity_list_tests",
                    "reason": "log_missing",
                },
            )
            return
        content = log_path.read_text(encoding="utf-8", errors="replace")
        test_count = 0
        for line in content.splitlines():
            if line.rstrip().endswith(": test"):
                test_count += 1
        metrics = {
            "generated_at": iso_now(),
            "rust_tests_discovered": test_count,
            "source_step": "sanity_list_tests",
        }
        write_json(metrics_path, metrics)

    def execute(self) -> int:
        """Run full flow and return process exit code."""
        self.setup_dirs()
        environment = self.collect_environment()
        steps = self.build_steps()
        for step in steps:
            if step.required or not self.quick:
                result = self.run_step(step)
                if not step.required and result.status == "fail":
                    result.status = "skip"
                    result.notes = f"Non-required step failed: {result.notes}"
                self.results.append(result)
                if self.fail_fast and result.required and result.status != "pass":
                    break

        practical_validation = self.run_practical_checks()
        self.results.append(practical_validation)
        pre_ai_summary = self.summarize()
        write_json(self.json_dir / "pre_ai_summary.json", pre_ai_summary)
        if not (self.text_dir / "ai_prompt.txt").exists():
            (self.text_dir / "ai_prompt.txt").write_text(
                "AI analyses were not run for this report.\n",
                encoding="utf-8",
            )
        if not (self.text_dir / "ai_codex.md").exists():
            (self.text_dir / "ai_codex.md").write_text("AI analysis not run.\n", encoding="utf-8")
        if not (self.text_dir / "ai_gemini.md").exists():
            (self.text_dir / "ai_gemini.md").write_text("AI analysis not run.\n", encoding="utf-8")

        if not self.skip_ai:
            self.run_ai_analyses(pre_ai_summary)

        summary = self.summarize()
        self.extract_test_count_metric()
        self.write_results_files(environment, summary)
        self.generate_readme(summary)
        self.create_latest_symlink()

        has_required_failure = len(summary["required_failures"]) > 0
        return 1 if has_required_failure else 0


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Run comprehensive tests and generate timestamped test report artifacts."
    )
    parser.add_argument(
        "--quick",
        action="store_true",
        help="Skip heavier required checks (clippy/integration/doc tests).",
    )
    parser.add_argument(
        "--skip-ai",
        action="store_true",
        help="Skip codex/gemini AI analyses.",
    )
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="Stop on first required failure.",
    )
    return parser.parse_args()


def main() -> int:
    """Entrypoint."""
    args = parse_args()
    runner = TestRunner(quick=args.quick, skip_ai=args.skip_ai, fail_fast=args.fail_fast)
    exit_code = runner.execute()
    report_rel = runner.report_dir.relative_to(ROOT).as_posix()
    print(f"Report: {report_rel}/README.md")
    print(f"Exit code: {exit_code}")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
