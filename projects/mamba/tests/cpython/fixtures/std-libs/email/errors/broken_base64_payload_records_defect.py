# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "broken_base64_payload_records_defect"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: a base64 body with a stray character decodes best-effort (returns the recoverable bytes) and records an InvalidBase64CharactersDefect in defects rather than raising"""
from email.message import Message
from email import errors

# A base64 body with a stray character ('9' breaking the alignment) decodes
# best-effort: get_payload(decode=True) returns the recoverable bytes and an
# InvalidBase64CharactersDefect is recorded rather than raising.
broken = Message()
broken["content-type"] = "audio/x-midi"
broken["content-transfer-encoding"] = "base64"
broken.set_payload("AwDp0P7//y6LwKEAcPa/6Q=9")
assert broken.get_payload(decode=True) == (
    b"\x03\x00\xe9\xd0\xfe\xff\xff.\x8b\xc0\xa1\x00p\xf6\xbf\xe9\x0f"
), broken.get_payload(decode=True)
assert isinstance(broken.defects[0], errors.InvalidBase64CharactersDefect), broken.defects

print("broken_base64_payload_records_defect OK")
