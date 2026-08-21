# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "all_exports_public_api"
# subject = "subprocess.__all__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.__all__: __all__ exports run and Popen but deliberately omits the low-level helper list2cmdline"""
import subprocess

assert "run" in subprocess.__all__, "run in __all__"
assert "Popen" in subprocess.__all__, "Popen in __all__"
assert "list2cmdline" not in subprocess.__all__, "list2cmdline excluded"
print("all_exports_public_api OK")
