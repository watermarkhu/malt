"""Smoke tests for the installed ``malt`` wheel."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from malt import find_malt_bin


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-m", "malt", *args],
        capture_output=True,
        text=True,
        check=False,
    )


def test_binary_is_bundled() -> None:
    assert Path(find_malt_bin()).is_file()


def test_version() -> None:
    result = run("--version")
    assert result.returncode == 0, result.stderr
    assert result.stdout.startswith("malt ")


def test_lints_a_file(tmp_path: Path) -> None:
    source = tmp_path / "script.m"
    source.write_text("x = 1\n")
    result = run(str(source))
    assert "NOPTS" in result.stdout, result.stdout + result.stderr


def test_clean_file_passes(tmp_path: Path) -> None:
    source = tmp_path / "addOne.m"
    source.write_text("function y = addOne(x)\ny = x + 1;\nend\n")
    result = run(str(source))
    assert result.returncode == 0, result.stdout + result.stderr
