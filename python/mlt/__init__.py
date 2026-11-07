"""mlt: an ultra-fast MATLAB linter.

The wheel ships the native ``mlt`` executable; this package only locates it so
that ``python -m mlt`` works in addition to the ``mlt`` console command.
"""

from mlt._find_mlt import find_mlt_bin

__all__ = ["find_mlt_bin"]
