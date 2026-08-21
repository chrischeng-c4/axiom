# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_cwd_sets_working_directory"
# subject = "subprocess.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.run: the cwd= keyword sets the child's working directory (compared via os.path.realpath against a TemporaryDirectory)"""
import os
import subprocess
import sys
import tempfile

with tempfile.TemporaryDirectory() as _tmpdir:
    _r = subprocess.run(
        [sys.executable, "-c", "import os; print(os.getcwd())"],
        capture_output=True, text=True, cwd=_tmpdir,
    )
    # On macOS /var/folders is /private/var/folders -> compare realpath.
    _got = os.path.realpath(_r.stdout.strip())
    _want = os.path.realpath(_tmpdir)
    assert _got == _want, f"cwd = {_got!r} vs {_want!r}"
print("run_cwd_sets_working_directory OK")
