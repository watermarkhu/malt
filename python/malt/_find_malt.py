"""Locate the ``malt`` executable installed alongside this package."""

from __future__ import annotations

import os
import sys
import sysconfig
from pathlib import Path


def find_malt_bin() -> str:
    """Return the path to the ``malt`` executable shipped with this package.

    Raises:
        FileNotFoundError: If the executable cannot be found.
    """
    exe = "malt.exe" if sys.platform == "win32" else "malt"

    candidates: list[Path] = [Path(sysconfig.get_path("scripts")) / exe]

    # `pip install --user` places scripts in the user scheme instead.
    user_scheme = {
        "nt": "nt_user",
        "posix": "osx_framework_user" if sys.platform == "darwin" else "posix_user",
    }.get(os.name)
    if user_scheme and user_scheme in sysconfig.get_scheme_names():
        candidates.append(Path(sysconfig.get_path("scripts", scheme=user_scheme)) / exe)

    # `pip install --target <dir>` puts scripts in <dir>/bin next to the package.
    candidates.append(Path(__file__).resolve().parent.parent / "bin" / exe)

    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)

    searched = ", ".join(str(c) for c in candidates)
    raise FileNotFoundError(
        f"could not find the malt executable (searched: {searched})"
    )
