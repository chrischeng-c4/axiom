# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "getstatusoutput_pairs_status_and_output"
# subject = "subprocess.getstatusoutput"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.getstatusoutput: subprocess.getstatusoutput returns (exit_status, output): zero status on success and a non-zero status for a failing shell command"""
import subprocess

status, output = subprocess.getstatusoutput("echo xyzzy")
assert (status, output) == (0, "xyzzy"), f"getstatusoutput = {(status, output)!r}"
status, _ = subprocess.getstatusoutput("exit 5")
assert status != 0, f"failing status = {status!r}"
print("getstatusoutput_pairs_status_and_output OK")
