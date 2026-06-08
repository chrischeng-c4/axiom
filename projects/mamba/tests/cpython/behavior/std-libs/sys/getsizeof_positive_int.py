# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "getsizeof_positive_int"
# subject = "sys.getsizeof"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getsizeof: getsizeof returns a positive int for both a small int and an empty list"""
import sys

_sz_int = sys.getsizeof(0)
_sz_list = sys.getsizeof([])
assert _sz_int > 0 and isinstance(_sz_int, int), f"int size = {_sz_int!r}"
assert _sz_list > 0 and isinstance(_sz_list, int), f"list size = {_sz_list!r}"
print("getsizeof_positive_int OK")
