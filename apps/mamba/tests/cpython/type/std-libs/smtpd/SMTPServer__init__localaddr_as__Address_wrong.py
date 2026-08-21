# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "SMTPServer__init__localaddr_as__Address_wrong"
# subject = "smtpd.SMTPServer.__init__(localaddr: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.SMTPServer.__init__(localaddr: _Address); call it with the wrong type.

typeshed contract: localaddr is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import SMTPServer
try:
    SMTPServer(_W(), None)  # localaddr: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
