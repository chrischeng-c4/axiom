# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "smtpd"
# dimension = "type"
# case = "PureProxy__process_message__peer_as__Address_wrong"
# subject = "smtpd.PureProxy.process_message(peer: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/smtpd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: smtpd.PureProxy.process_message(peer: _Address); call it with the wrong type.

typeshed contract: peer is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from smtpd import PureProxy
obj = object.__new__(PureProxy)
try:
    obj.process_message(_W(), "", None, None)  # peer: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
