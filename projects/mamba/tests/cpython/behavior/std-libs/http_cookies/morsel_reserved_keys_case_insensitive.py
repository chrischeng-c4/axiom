# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_reserved_keys_case_insensitive"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell is not a real dict; reserved keys and case-insensitive lookup are absent (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.Morsel: a fresh Morsel exposes the reserved attribute keys with empty defaults; reserved-key lookup/assignment is case-insensitive"""
from http import cookies

m = cookies.Morsel()
assert m.key is None, f"key = {m.key!r}"
assert m.value is None, f"value = {m.value!r}"
assert m.coded_value is None, f"coded_value = {m.coded_value!r}"
assert m.keys() == cookies.Morsel._reserved.keys(), "keys match reserved set"
for k, v in m.items():
    assert v == "", f"default attr {k!r} = {v!r}"
m["Version"] = 2
assert m["version"] == 2, f"version = {m['version']!r}"
m["DOMAIN"] = "example.com"
assert m["domain"] == "example.com", f"domain = {m['domain']!r}"
print("morsel_reserved_keys_case_insensitive OK")
