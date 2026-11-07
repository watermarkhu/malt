"""Run the bundled ``mlt`` executable: ``python -m mlt [args...]``."""

from __future__ import annotations

import os
import subprocess
import sys

from mlt._find_mlt import find_mlt_bin


def main() -> None:
    """Replace this process with ``mlt`` (or wait for it on Windows)."""
    mlt = find_mlt_bin()
    if sys.platform == "win32":
        # Windows has no exec(); forward the exit code instead.
        sys.exit(subprocess.run([mlt, *sys.argv[1:]]).returncode)
    os.execvp(mlt, [mlt, *sys.argv[1:]])


if __name__ == "__main__":
    main()
