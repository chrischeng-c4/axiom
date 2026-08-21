# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "version_info_attributes"
# subject = "sys.version_info"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.version_info: version_info exposes major==3, non-negative minor/micro, and a valid releaselevel in (alpha, beta, candidate, final)"""
import sys

assert sys.version_info.major == 3, f"major = {sys.version_info.major!r}"
assert sys.version_info.minor >= 0, f"minor = {sys.version_info.minor!r}"
assert sys.version_info.micro >= 0, f"micro = {sys.version_info.micro!r}"
assert sys.version_info.releaselevel in ("alpha", "beta", "candidate", "final"), \
    f"releaselevel = {sys.version_info.releaselevel!r}"
print("version_info_attributes OK")
