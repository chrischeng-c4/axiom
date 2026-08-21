# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "reach_broadest_safe_domain"
# subject = "http.cookiejar.reach"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.reach: reach returns the broadest domain a host may safely set cookies for (www.acme.com -> .acme.com, acme.com -> acme.com, IP -> itself)"""
from http.cookiejar import reach

assert reach("www.acme.com") == ".acme.com"
assert reach("acme.com") == "acme.com"
assert reach("acme.local") == ".local"
assert reach(".local") == ".local"
assert reach("192.168.0.1") == "192.168.0.1"

print("reach_broadest_safe_domain OK")
