"""malt: an ultra-fast MATLAB linter.

The wheel ships the native ``malt`` executable; this package only locates it so
that ``python -m malt`` works in addition to the ``malt`` console command.
"""

from malt._find_malt import find_malt_bin

__all__ = ["find_malt_bin"]
