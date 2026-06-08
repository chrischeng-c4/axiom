# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "b2a_qp_header_and_quotetabs"
# subject = "binascii.b2a_qp"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.b2a_qp: b2a_qp header mode maps space to underscore; quotetabs escapes whitespace"""
import binascii

assert binascii.b2a_qp(b"_", header=True) == b"=5F", "underscore escaped in header"
assert binascii.b2a_qp(b"x y", header=True) == b"x_y", "space -> underscore"
assert binascii.b2a_qp(b"x y\tz", quotetabs=True) == b"x=20y=09z", "quotetabs escapes ws"

print("b2a_qp_header_and_quotetabs OK")
