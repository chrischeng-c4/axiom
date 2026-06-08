# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_full_metadata_and_js_output"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output()/js_output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: load() captures Version/Path metadata, output([attr]) filters, and js_output() emits the fixed <script> document.cookie wrapper"""
from http import cookies

c = cookies.SimpleCookie()
c.load('Customer="WILE_E_COYOTE"; Version=1; Path=/acme')
assert c["Customer"]["version"] == "1", f"version = {c['Customer']['version']!r}"
assert c.output(["path"]) == 'Set-Cookie: Customer="WILE_E_COYOTE"; Path=/acme', \
    f"filtered output = {c.output(['path'])!r}"
expected_js = (
    '\n        <script type="text/javascript">'
    '\n        <!-- begin hiding'
    '\n        document.cookie = "Customer=\\"WILE_E_COYOTE\\"; Path=/acme; Version=1";'
    '\n        // end hiding -->'
    '\n        </script>\n        '
)
assert c.js_output() == expected_js, f"js_output = {c.js_output()!r}"
print("load_full_metadata_and_js_output OK")
