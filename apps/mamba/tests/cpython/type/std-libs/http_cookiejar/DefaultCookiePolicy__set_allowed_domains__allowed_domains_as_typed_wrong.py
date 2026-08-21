# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_allowed_domains__allowed_domains_as_typed_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_allowed_domains(allowed_domains: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allowed_domains"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed allowed_domains
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_allowed_domains(allowed_domains: typed); call it with the wrong type.

typeshed contract: allowed_domains is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_allowed_domains(_W())  # allowed_domains: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
