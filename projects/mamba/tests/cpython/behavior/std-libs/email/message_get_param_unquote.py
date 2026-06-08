# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_get_param_unquote"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_param unquotes a quoted Content-Type parameter by default; unquote=False keeps the surrounding quotes; a bare value-less attribute yields an empty-string param"""
import email

pm = email.message_from_string('Content-Type: image/pjpeg; name="A&&B"\n')
assert pm.get_param("name") == "A&&B", pm.get_param("name")
assert pm.get_param("name", unquote=False) == '"A&&B"', pm.get_param("name", unquote=False)

# A bare attribute with no value yields an empty-string param value.
bp = email.message_from_string("Content-Type: blarg; baz; boo\n")
assert bp.get_param("baz") == "", repr(bp.get_param("baz"))

print("message_get_param_unquote OK")
