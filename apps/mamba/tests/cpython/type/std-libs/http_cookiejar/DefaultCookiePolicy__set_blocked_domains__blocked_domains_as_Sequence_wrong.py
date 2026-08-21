# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_blocked_domains__blocked_domains_as_Sequence_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_blocked_domains(blocked_domains: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocked_domains"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed blocked_domains
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_blocked_domains(blocked_domains: Sequence); call it with the wrong type.

typeshed contract: blocked_domains is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_blocked_domains(_W())  # blocked_domains: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
