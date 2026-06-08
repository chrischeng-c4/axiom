# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "behavior"
# case = "get_cache_token_returns_int"
# subject = "abc.get_cache_token"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""abc.get_cache_token: get_cache_token() returns an int per the CPython contract"""
import abc

tok = abc.get_cache_token()
assert isinstance(tok, int), f"get_cache_token returns int, got {type(tok).__name__}"

print("get_cache_token_returns_int OK")
