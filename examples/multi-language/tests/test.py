from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import pytest

TEST_DIR = Path(__file__).resolve().parent
EXAMPLE_DIR = TEST_DIR.parent
EXPORT_DIR = EXAMPLE_DIR / "export"
IMPORT_DIR = EXAMPLE_DIR / "import"
REPO_ROOT = TEST_DIR.parents[2]
PYTHON_LIB_DIR = REPO_ROOT / "library" / "python"
EXPECTED_OUTPUT = (TEST_DIR / "data" / "kv_store_expected.txt").read_text().splitlines()

EXPORT_LANGUAGES = tuple(
    sorted(
        directory.name
        for directory in EXPORT_DIR.iterdir()
        if directory.is_dir() and (directory / "Makefile").exists()
    )
)
IMPORT_LANGUAGES = tuple(
    sorted(
        directory.name
        for directory in IMPORT_DIR.iterdir()
        if directory.is_dir() and (directory / "Makefile").exists()
    )
)
EXPORT_PARAM_IDS = tuple(f"export_{language}" for language in EXPORT_LANGUAGES)
IMPORT_PARAM_IDS = tuple(f"import_{language}" for language in IMPORT_LANGUAGES)


def run_make(directory: Path, *args: str) -> None:
    """Invoke ``make`` inside ``directory`` with the provided arguments."""

    subprocess.run(["make", *args], cwd=directory, check=True)


def capture_stdout(command: list[str], *, cwd: Path, env: dict[str, str] | None = None) -> list[str]:
    """Execute ``command`` and return its stdout split into individual lines."""

    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return [line.rstrip("\r") for line in result.stdout.splitlines()]


@pytest.mark.parametrize("export_language", EXPORT_LANGUAGES, ids=EXPORT_PARAM_IDS)
@pytest.mark.parametrize("import_language", IMPORT_LANGUAGES, ids=IMPORT_PARAM_IDS)
def test(export_language: str, import_language: str) -> None:
    """Every import implementation should match the shared golden output."""

    run_make(EXPORT_DIR / export_language, "lib")

    if import_language == "python":
        run_make(IMPORT_DIR / "python", "bindings")
        env = os.environ.copy()
        env["PYTHONPATH"] = str(PYTHON_LIB_DIR)
        output = capture_stdout(
            [sys.executable, "main.py", f"--library={export_language}"],
            cwd=IMPORT_DIR / "python",
            env=env,
        )
    else:
        run_make(IMPORT_DIR / import_language, f"EXPORT={export_language}", "binary")
        binary = f"kv_store_from_{import_language}_against_{export_language}"
        output = capture_stdout([f"./{binary}"], cwd=IMPORT_DIR / import_language)

    assert output == EXPECTED_OUTPUT
