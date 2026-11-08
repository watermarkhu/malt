"""Run the bundled ``malt`` executable: ``python -m malt [args...]``."""

from __future__ import annotations

import os
import subprocess
import sys

from malt._find_malt import find_malt_bin


def main() -> None:
    """Replace this process with ``malt`` (or wait for it on Windows)."""
    malt = find_malt_bin()
    if sys.platform == "win32":
        # Windows has no exec(); forward the exit code instead.
        sys.exit(subprocess.run([malt, *sys.argv[1:]]).returncode)
    os.execvp(malt, [malt, *sys.argv[1:]])


if __name__ == "__main__":
    main()
