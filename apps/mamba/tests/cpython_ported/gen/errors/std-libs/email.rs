use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/email/broken_base64_payload_records_defect.py`.
#[test]
fn test_gen_errors_std_libs_email_broken_base64_payload_records_defect() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"broken_base64_payload_records_defect OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/email/get_payload_index_on_non_multipart_raises.py`.
#[test]
fn test_gen_errors_std_libs_email_get_payload_index_on_non_multipart_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "get_payload_index_on_non_multipart_raises"
# subject = "email.message.Message"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_payload_index_on_non_multipart_raises (errors)."""
from email.message import Message

_raised = False
try:
    Message().get_payload(1)
except TypeError:
    _raised = True
assert _raised, "get_payload_index_on_non_multipart_raises: expected TypeError"
print("get_payload_index_on_non_multipart_raises OK")
"###);
    assert_output(&out, r###"get_payload_index_on_non_multipart_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/email/header_bad_charset_raises.py`.
#[test]
fn test_gen_errors_std_libs_email_header_bad_charset_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "header_bad_charset_raises"
# subject = "email.header.Header"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.header.Header: header_bad_charset_raises (errors)."""
from email.header import Header

_raised = False
try:
    Header("hi", charset="no_such_charset")
except LookupError:
    _raised = True
assert _raised, "header_bad_charset_raises: expected LookupError"
print("header_bad_charset_raises OK")
"###);
    assert_output(&out, r###"header_bad_charset_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/email/long_header_serializes_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_email_long_header_serializes_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "long_header_serializes_no_raise"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: an oversized 10000-char header value is accepted by the default policy; as_string() folds and serializes it without raising HeaderParseError"""
from email.message import EmailMessage

# The default policy accepts a very long header value and folds it on
# serialization; no HeaderParseError is raised.
m = EmailMessage()
m["From"] = "a" * 10000
s = m.as_string()
assert len(s) > 0, "serialized output should be non-empty"
assert "From:" in s, "From header should survive serialization"

print("long_header_serializes_no_raise OK")
"###);
    assert_output(&out, r###"long_header_serializes_no_raise OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/email/malformed_header_collects_defect_no_raise.py`.
#[test]
fn test_gen_errors_std_libs_email_malformed_header_collects_defect_no_raise() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "malformed_header_collects_defect_no_raise"
# subject = "email.message_from_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message_from_string: parsing a header line without a colon under the default policy does NOT raise; the defect is collected in msg.defects (non-empty) instead"""
from email import message_from_string

# A header line with no colon is malformed. Under the default policy the parser
# does NOT raise; it records the defect on msg.defects instead.
msg = message_from_string("not_a_valid_header without colon\n")
assert len(msg.defects) >= 1, f"expected a collected defect, got {msg.defects!r}"

print("malformed_header_collects_defect_no_raise OK")
"###);
    assert_output(&out, r###"malformed_header_collects_defect_no_raise OK
"###);
}
