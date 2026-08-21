# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "separate_lambdas_distinct"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: two separately compiled `lambda: 1` expressions produce distinct, unequal code objects"""
import types

ca = (lambda: 1).__code__
cb = (lambda: 1).__code__
assert ca is not cb, "separately compiled lambdas are distinct objects"
assert ca != cb, "separately compiled lambdas compare unequal (differing firstlineno)"

print("separate_lambdas_distinct OK")
