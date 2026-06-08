# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "getoutput_returns_trimmed_stdout"
# subject = "subprocess.getoutput"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.getoutput: subprocess.getoutput runs a shell command and returns its trimmed stdout text"""
import subprocess

assert subprocess.getoutput("echo xyzzy") == "xyzzy", "getoutput"
print("getoutput_returns_trimmed_stdout OK")
