# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_copy_is_equal_but_distinct"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound copy() and no value equality (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.Morsel: Morsel.copy() and copy.copy() return an equal-but-distinct Morsel"""
from http import cookies

import copy

m = cookies.Morsel()
m.set("foo", "bar", "baz")
m.update({"version": 2, "comment": "foo"})
for dup in (m.copy(), copy.copy(m)):
    assert isinstance(dup, cookies.Morsel), "copy is a Morsel"
    assert dup is not m and dup == m, "copy is distinct but equal"
print("morsel_copy_is_equal_but_distinct OK")
