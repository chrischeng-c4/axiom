# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__init__blocked_domains_as_typed_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.__init__(blocked_domains: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocked_domains"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocked_domains
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.__init__(blocked_domains: typed); call it with the wrong type.

typeshed contract: blocked_domains is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
try:
    DefaultCookiePolicy(_W())  # blocked_domains: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
