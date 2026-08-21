# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLContext__load_default_certs__purpose_as_Purpose_wrong"
# subject = "ssl.SSLContext.load_default_certs(purpose: Purpose)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLContext.load_default_certs(purpose: Purpose); call it with the wrong type.

typeshed contract: purpose is Purpose. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ssl import SSLContext
obj = object.__new__(SSLContext)
try:
    obj.load_default_certs(_W())  # purpose: Purpose <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
