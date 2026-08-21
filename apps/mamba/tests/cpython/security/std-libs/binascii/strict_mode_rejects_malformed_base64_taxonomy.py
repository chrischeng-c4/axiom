# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "security"
# case = "strict_mode_rejects_malformed_base64_taxonomy"
# subject = "binascii.a2b_base64"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.a2b_base64: strict_mode=True classifies malformed base64 (excess/leading/discontinuous/excess padding); lenient mode decodes the valid prefix"""
import binascii


def strict_category(data):
    """Return the lowercased binascii.Error message for strict decoding."""
    try:
        binascii.a2b_base64(data, strict_mode=True)
        return "no_raise"
    except binascii.Error as e:
        return str(e).lower()


# Each malformed input maps to a distinct strict-mode complaint.
_cases = {
    b"ab==a": "excess data",         # trailing bytes after the padding
    b"\nab==": "only base64 data",   # leading non-base64 byte
    b"=": "leading padding",         # padding with nothing before it
    b"ab=c=": "discontinuous",       # data char between padding chars
    b"abcd=": "excess padding",      # more padding than the group needs
}
for data, needle in _cases.items():
    msg = strict_category(data)
    assert needle in msg, f"{data!r}: expected {needle!r} in {msg!r}"

# Non-strict mode tolerates the same inputs, decoding the valid prefix.
assert binascii.a2b_base64(b"ab==a") == b"i", "lenient excess data"
assert binascii.a2b_base64(b"\nab==") == b"i", "lenient leading noise"
assert binascii.a2b_base64(b"=") == b"", "lenient leading padding"
# strict_mode=False is identical to the default (no keyword).
assert binascii.a2b_base64(b"abcd=", strict_mode=False) == binascii.a2b_base64(b"abcd="), \
    "strict_mode=False == default"

print("strict_mode_rejects_malformed_base64_taxonomy OK")
