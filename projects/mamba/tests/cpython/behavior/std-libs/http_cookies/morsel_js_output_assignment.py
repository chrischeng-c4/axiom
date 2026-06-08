# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_js_output_assignment"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound js_output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: Morsel.js_output() emits a JavaScript document.cookie assignment carrying the key and value"""
from http import cookies

c = cookies.SimpleCookie()
c["js_key"] = "js_val"
js = c["js_key"].js_output()
assert isinstance(js, str), f"js_output type = {type(js)!r}"
assert "js_key" in js, f"key in js_output: {js!r}"
assert "js_val" in js, f"value in js_output: {js!r}"
print("morsel_js_output_assignment OK")
