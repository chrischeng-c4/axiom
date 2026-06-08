# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_setdefault_and_update"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound setdefault()/update() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.Morsel: Morsel.setdefault keeps existing reserved values and sets defaults; update accepts a dict or pair list of reserved keys"""
from http import cookies

m = cookies.Morsel()
m.update({"domain": "example.com", "version": 2})
assert m.setdefault("expires", "value") == "", "default expires was empty"
assert m.setdefault("Version", 1) == 2, "existing version kept"
assert m.setdefault("DOMAIN", "value") == "example.com", "existing domain kept"
m2 = cookies.Morsel()
m2.update(list({"expires": 1, "Version": 2}.items()))
assert m2["expires"] == 1 and m2["version"] == 2, "update from pair list"
print("morsel_setdefault_and_update OK")
