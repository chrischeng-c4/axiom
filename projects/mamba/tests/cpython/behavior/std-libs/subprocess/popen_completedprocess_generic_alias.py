# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_completedprocess_generic_alias"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: Popen[bytes] and CompletedProcess[str] subscription each yield a types.GenericAlias"""
import subprocess
import types

assert isinstance(subprocess.Popen[bytes], types.GenericAlias), "Popen[bytes]"
assert isinstance(
    subprocess.CompletedProcess[str], types.GenericAlias
), "CompletedProcess[str]"
print("popen_completedprocess_generic_alias OK")
