# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_accepts_pathlike_args"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: an os.PathLike executable (custom __fspath__) is accepted just like a str path; the child exit code propagates to returncode"""
import subprocess
import sys


class _FakePath:
    """Minimal os.PathLike wrapper to test path-like argument acceptance."""

    def __init__(self, value):
        self._value = value

    def __fspath__(self):
        return self._value


_r = subprocess.run([_FakePath(sys.executable), "-c", "import sys; sys.exit(57)"])
assert _r.returncode == 57, f"pathlike rc = {_r.returncode!r}"
print("run_accepts_pathlike_args OK")
