# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "protocol_member_is_intenum_like"
# subject = "ssl.PROTOCOL_TLS_CLIENT"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.PROTOCOL_TLS_CLIENT: a protocol constant behaves like an IntEnum member: repr names it as <_SSLMethod.PROTOCOL_TLS_CLIENT: value>, str is its int value, int() coerces to its value, and PROTOCOL_TLS aliases PROTOCOL_SSLv23"""
import ssl

_proto = ssl.PROTOCOL_TLS_CLIENT
assert repr(_proto) == "<_SSLMethod.PROTOCOL_TLS_CLIENT: %r>" % _proto.value, \
    f"repr = {_proto!r}"
assert str(_proto) == str(_proto.value), f"str = {_proto}"
assert int(_proto) == _proto.value, "protocol coerces to int"
assert ssl.PROTOCOL_TLS == ssl.PROTOCOL_SSLv23, "PROTOCOL_TLS == PROTOCOL_SSLv23"

print("protocol_member_is_intenum_like OK")
