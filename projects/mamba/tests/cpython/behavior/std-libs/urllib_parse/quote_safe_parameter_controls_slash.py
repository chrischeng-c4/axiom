# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "quote_safe_parameter_controls_slash"
# subject = "urllib.parse.quote"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote leaves the default safe '/' alone, safe='' escapes every reserved char including '/', and a custom safe='=' keeps '=' while still escaping '+'"""
from urllib.parse import quote

assert quote("/dir/file.html") == "/dir/file.html", "default safe '/' preserved"
assert quote("/dir/file.html", safe="") == "%2Fdir%2Ffile.html", "safe='' encodes slashes"
assert quote("hello world") == "hello%20world", "space always escaped"
assert quote("a+b=c", safe="=") == "a%2Bb=c", "safe='=' keeps =, escapes +"

print("quote_safe_parameter_controls_slash OK")
