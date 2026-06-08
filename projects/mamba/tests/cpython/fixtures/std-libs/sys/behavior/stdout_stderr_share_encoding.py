# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "stdout_stderr_share_encoding"
# subject = "sys.__stdout__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.__stdout__: the original __stdout__ and __stderr__ streams report the same encoding"""
import sys

assert sys.__stdout__.encoding == sys.__stderr__.encoding, \
    f"stdout enc {sys.__stdout__.encoding!r} != stderr enc {sys.__stderr__.encoding!r}"
print("stdout_stderr_share_encoding OK")
