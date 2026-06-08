# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "is_hdn_host_domain_name"
# subject = "http.cookiejar.is_HDN"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.is_HDN: is_HDN is True only for a genuine host domain name, False for IP literals, empty, '.', leading-dot, or trailing-dot forms"""
from http.cookiejar import is_HDN

assert is_HDN("foo.bar.com")
assert is_HDN("1foo2.3bar4.5com")
assert not is_HDN("192.168.1.1")
assert not is_HDN("")
assert not is_HDN(".")
assert not is_HDN(".foo.bar.com")
assert not is_HDN("foo.")

print("is_hdn_host_domain_name OK")
