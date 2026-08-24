use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/email/bytesparser_parsebytes_recovers_headers.py`.
#[test]
fn test_gen_behavior_std_libs_email_bytesparser_parsebytes_recovers_headers() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "bytesparser_parsebytes_recovers_headers"
# subject = "email.parser.BytesParser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.parser.BytesParser: BytesParser().parsebytes parses a raw bytes message and recovers the From and Subject headers"""
from email.parser import BytesParser

bp = BytesParser()
raw = b"From: test@example.com\r\nSubject: Bytes\r\n\r\nByte body.\r\n"
m = bp.parsebytes(raw)
assert m["From"] == "test@example.com", f"BytesParser From = {m['From']!r}"
assert m["Subject"] == "Bytes", f"BytesParser Subject = {m['Subject']!r}"

print("bytesparser_parsebytes_recovers_headers OK")
"###);
    assert_output(&out, r###"bytesparser_parsebytes_recovers_headers OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/decode_header_splits_chunks.py`.
#[test]
fn test_gen_behavior_std_libs_email_decode_header_splits_chunks() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "decode_header_splits_chunks"
# subject = "email.header.decode_header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.decode_header: decode_header splits an encoded word into (bytes, charset) chunks; a word mixed with trailing text reports both, the unencoded run carrying a None charset"""
from email.header import decode_header

# decode_header splits an encoded word into (bytes, charset) chunks.
dec = decode_header("=?utf-8?q?foo?=")
assert dec == [(b"foo", "utf-8")], dec

# A header mixing an encoded word and trailing text reports both chunks,
# with a None charset for the unencoded run.
mixed = decode_header("=?utf-8?b?Zm9v?= plain")
assert mixed == [(b"foo", "utf-8"), (b" plain", None)], mixed

print("decode_header_splits_chunks OK")
"###);
    assert_output(&out, r###"decode_header_splits_chunks OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/emailmessage_as_bytes_rfc5322.py`.
#[test]
fn test_gen_behavior_std_libs_email_emailmessage_as_bytes_rfc5322() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_as_bytes_rfc5322"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: EmailMessage.as_bytes() returns a bytes serialization carrying the From header and the body bytes"""
from email.message import EmailMessage

msg = EmailMessage()
msg["From"] = "sender@example.com"
msg.set_content("Bytes body")
b = msg.as_bytes()
assert isinstance(b, bytes), f"as_bytes type = {type(b)!r}"
assert b"From:" in b, "From in bytes"
assert b"Bytes body" in b, "body in bytes"

print("emailmessage_as_bytes_rfc5322 OK")
"###);
    assert_output(&out, r###"emailmessage_as_bytes_rfc5322 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/emailmessage_as_string_rfc5322.py`.
#[test]
fn test_gen_behavior_std_libs_email_emailmessage_as_string_rfc5322() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_as_string_rfc5322"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: EmailMessage with From/To/Subject and set_content serializes via as_string() to text carrying the headers and the body"""
from email.message import EmailMessage

msg = EmailMessage()
msg["From"] = "alice@example.com"
msg["To"] = "bob@example.com"
msg["Subject"] = "Hello"
msg.set_content("Test body here.")
s = msg.as_string()
assert "From:" in s, f"From header present: {s[:80]!r}"
assert "Subject: Hello" in s, f"Subject present: {s[:200]!r}"
assert "Test body here." in s, "body present"

print("emailmessage_as_string_rfc5322 OK")
"###);
    assert_output(&out, r###"emailmessage_as_string_rfc5322 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/emailmessage_duplicate_header_kept.py`.
#[test]
fn test_gen_behavior_std_libs_email_emailmessage_duplicate_header_kept() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "emailmessage_duplicate_header_kept"
# subject = "email.message.EmailMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.EmailMessage: setting the same header name twice keeps both occurrences; items() reports the duplicate name twice"""
from email.message import EmailMessage

msg = EmailMessage()
msg["X-Header-One"] = "val1"
msg["X-Header-Two"] = "val2"
msg["X-Header-One"] = "val3"  # duplicate name
hdr_names = [k for k, v in msg.items()]
assert hdr_names.count("X-Header-One") == 2, f"duplicate headers: {hdr_names!r}"

print("emailmessage_duplicate_header_kept OK")
"###);
    assert_output(&out, r###"emailmessage_duplicate_header_kept OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/header_encode_ascii_qword.py`.
#[test]
fn test_gen_behavior_std_libs_email_header_encode_ascii_qword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "header_encode_ascii_qword"
# subject = "email.header.Header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.Header: a short ascii-safe utf-8 value encodes to a q-encoded word (=?utf-8?q?foo?=); a pure-ascii header with no charset stays unencoded"""
from email.header import Header

# A short ascii-safe utf-8 value is emitted as a q-encoded word.
assert Header("foo", charset="utf-8").encode() == "=?utf-8?q?foo?=", Header(
    "foo", charset="utf-8"
).encode()

# A pure-ascii header with no charset stays unencoded.
assert Header("plain ascii").encode() == "plain ascii", Header("plain ascii").encode()

print("header_encode_ascii_qword OK")
"###);
    assert_output(&out, r###"header_encode_ascii_qword OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/header_encode_non_ascii_base64.py`.
#[test]
fn test_gen_behavior_std_libs_email_header_encode_non_ascii_base64() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "header_encode_non_ascii_base64"
# subject = "email.header.Header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.Header: non-ascii content (caña) encodes to a base64 encoded word (=?utf-8?b?Y2HDsWE=?=)"""
from email.header import Header

# Non-ascii content is base64-encoded as an encoded word.
ena = Header("ca\xf1a", charset="utf-8").encode()
assert ena == "=?utf-8?b?Y2HDsWE=?=", ena

print("header_encode_non_ascii_base64 OK")
"###);
    assert_output(&out, r###"header_encode_non_ascii_base64 OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/make_header_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_email_make_header_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "make_header_roundtrip"
# subject = "email.header.make_header"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test__encoded_words.py"
# status = "filled"
# ///
"""email.header.make_header: make_header(decode_header(...)) reassembles decoded chunks back to text; a full encode->decode->make_header round-trip recovers the source string (Grüße)"""
from email.header import Header, decode_header, make_header

# make_header reassembles decoded chunks back into the original text.
assert str(make_header(decode_header("=?utf-8?b?Y2HDsWE=?="))) == "ca\xf1a", str(
    make_header(decode_header("=?utf-8?b?Y2HDsWE=?="))
)

# Round-trip: encode then decode recovers the source string.
src = "Gr\xfc\xdfe"
recovered = str(make_header(decode_header(Header(src, charset="utf-8").encode())))
assert recovered == src, recovered

print("make_header_roundtrip OK")
"###);
    assert_output(&out, r###"make_header_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_content_disposition_normalized.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_content_disposition_normalized() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_content_disposition_normalized"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_content_disposition() lower-cases the disposition token while the raw header value keeps its spelling; add_header/replace_header and get_filename follow suit"""
from email.message import Message

disp = Message()
assert disp.get_content_disposition() is None, "no disposition yet"
disp.add_header("Content-Disposition", "attachment", filename="random.avi")
assert disp.get_content_disposition() == "attachment", disp.get_content_disposition()
assert disp.get_filename() == "random.avi", disp.get_filename()
disp.replace_header("Content-Disposition", "InlinE")
assert disp.get_content_disposition() == "inline", disp.get_content_disposition()
assert disp["content-disposition"] == "InlinE", disp["content-disposition"]

print("message_content_disposition_normalized OK")
"###);
    assert_output(&out, r###"message_content_disposition_normalized OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_default_content_type_text_plain.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_default_content_type_text_plain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_default_content_type_text_plain"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: a bare Message with no Content-Type defaults to text/plain: get_content_type/maintype/subtype return text/plain, text, plain"""
from email.message import Message

empty = Message()
assert empty.get_content_type() == "text/plain", empty.get_content_type()
assert empty.get_content_maintype() == "text", empty.get_content_maintype()
assert empty.get_content_subtype() == "plain", empty.get_content_subtype()

print("message_default_content_type_text_plain OK")
"###);
    assert_output(&out, r###"message_default_content_type_text_plain OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_del_param_keeps_bare_value.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_del_param_keeps_bare_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_del_param_keeps_bare_value"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: del_param drops a single param (filename) but keeps the bare disposition value (attachment)"""
from email.message import Message

disp = Message()
disp.add_header("Content-Disposition", "attachment", filename="bud.gif")
disp.del_param("filename", "content-disposition")
assert disp["content-disposition"] == "attachment", disp["content-disposition"]

print("message_del_param_keeps_bare_value OK")
"###);
    assert_output(&out, r###"message_del_param_keeps_bare_value OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_from_bytes_adds_implicit_cte.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_from_bytes_adds_implicit_cte() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_from_bytes_adds_implicit_cte"
# subject = "email.message_from_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message_from_bytes: a non-ascii iso-8859-1 body with no declared Content-Transfer-Encoding gains a quoted-printable CTE on as_string(), encoding the high bytes (F=F6=F6 b=E4r)"""
import email

import textwrap

# A non-ascii iso-8859-1 body with no declared Content-Transfer-Encoding gains
# a quoted-printable CTE when re-serialized to a string.
src = textwrap.dedent('''\
MIME-Version: 1.0
Content-type: text/plain; charset="iso-8859-1"

Non-ascii body: F\xf6\xf6 b\xe4r
''').encode("iso-8859-1")
m = email.message_from_bytes(src)
out = m.as_string()
assert "Content-Transfer-Encoding: quoted-printable" in out, "implicit CTE added"
assert "F=F6=F6 b=E4r" in out, f"quoted-printable body: {out[-40:]!r}"

print("message_from_bytes_adds_implicit_cte OK")
"###);
    assert_output(&out, r###"message_from_bytes_adds_implicit_cte OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_get_filename_missing_vs_empty.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_get_filename_missing_vs_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_get_filename_missing_vs_empty"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_filename returns None when no filename is present, but returns '' for a value-less filename param"""
import email

assert email.message_from_string("From: foo\n").get_filename() is None, "no filename"
bogus = email.message_from_string("Content-Disposition: blarg; filename\n")
assert bogus.get_filename() == "", repr(bogus.get_filename())

print("message_get_filename_missing_vs_empty OK")
"###);
    assert_output(&out, r###"message_get_filename_missing_vs_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_get_param_unquote.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_get_param_unquote() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"message_get_param_unquote OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_header_membership_case_insensitive.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_header_membership_case_insensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_header_membership_case_insensitive"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: header membership (`in`) is case-insensitive in both directions; an absent header reports not-in"""
from email.message import Message

m = Message()
m["From"] = "Me"
m["to"] = "You"
for probe in ("from", "From", "FROM", "to", "To", "TO"):
    assert probe in m, f"{probe!r} should be in message"
assert "missing" not in m, "absent header reports not-in"

print("message_header_membership_case_insensitive OK")
"###);
    assert_output(&out, r###"message_header_membership_case_insensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/message_set_type_rewrites_token.py`.
#[test]
fn test_gen_behavior_std_libs_email_message_set_type_rewrites_token() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_set_type_rewrites_token"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_type rewrites only the type token of an arbitrary named header, leaving its other params intact"""
from email.message import Message

st = Message()
st["X-Content-Type"] = "text/plain"
st.set_type("application/octet-stream", "X-Content-Type")
assert st["x-content-type"] == "application/octet-stream", st["x-content-type"]

print("message_set_type_rewrites_token OK")
"###);
    assert_output(&out, r###"message_set_type_rewrites_token OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/mimemultipart_attach_and_walk.py`.
#[test]
fn test_gen_behavior_std_libs_email_mimemultipart_attach_and_walk() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "mimemultipart_attach_and_walk"
# subject = "email.mime.multipart.MIMEMultipart"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.mime.multipart.MIMEMultipart: MIMEMultipart('alternative') with attached plain+html parts: get_content_maintype()=='multipart', payload has the parts, and walk() yields the container plus both text/plain and text/html parts"""
from email.mime.multipart import MIMEMultipart

from email.mime.text import MIMEText

multi = MIMEMultipart("alternative")
multi.attach(MIMEText("Plain text", "plain"))
multi.attach(MIMEText("<b>HTML</b>", "html"))
assert multi.get_content_maintype() == "multipart", "multipart maintype"
payload = multi.get_payload()
assert len(payload) == 2, f"two parts = {len(payload)!r}"
parts = list(multi.walk())
# walk yields the multipart container itself plus the two parts.
assert len(parts) >= 3, f"walk parts = {len(parts)!r}"
ctypes = [p.get_content_type() for p in parts]
assert "text/plain" in ctypes, "plain text part"
assert "text/html" in ctypes, "html part"

print("mimemultipart_attach_and_walk OK")
"###);
    assert_output(&out, r###"mimemultipart_attach_and_walk OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/mimetext_content_type_and_charset.py`.
#[test]
fn test_gen_behavior_std_libs_email_mimetext_content_type_and_charset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "mimetext_content_type_and_charset"
# subject = "email.mime.text.MIMEText"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.mime.text.MIMEText: MIMEText(body, subtype, charset) reports get_content_type() == 'text/<subtype>' and get_content_charset() reflecting the declared charset (utf-8), with None/str when omitted"""
from email.mime.text import MIMEText

mime = MIMEText("Plain text content", "plain", "utf-8")
assert isinstance(mime, MIMEText), "MIMEText type"
assert mime.get_content_type() == "text/plain", f"type = {mime.get_content_type()!r}"
assert mime.get_content_charset() == "utf-8", f"charset = {mime.get_content_charset()!r}"

# Without an explicit charset the charset is either None or a str (us-ascii).
plain = MIMEText("hello", "plain")
cs = plain.get_content_charset()
assert cs is None or isinstance(cs, str), f"charset type = {type(cs)!r}"

print("mimetext_content_type_and_charset OK")
"###);
    assert_output(&out, r###"mimetext_content_type_and_charset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/parser_parsestr_roundtrips_headers_and_body.py`.
#[test]
fn test_gen_behavior_std_libs_email_parser_parsestr_roundtrips_headers_and_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "parser_parsestr_roundtrips_headers_and_body"
# subject = "email.parser.Parser"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.parser.Parser: Parser().parsestr round-trips a raw RFC 5322 message: From/Subject headers and the body text are recovered"""
from email.parser import Parser

p = Parser()
original = "From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Parse\r\n\r\nBody text.\r\n"
m = p.parsestr(original)
assert m["From"] == "alice@example.com", f"parsed From = {m['From']!r}"
assert m["Subject"] == "Parse", f"parsed Subject = {m['Subject']!r}"
assert "Body text." in m.get_payload(), f"parsed body = {m.get_payload()!r}"

print("parser_parsestr_roundtrips_headers_and_body OK")
"###);
    assert_output(&out, r###"parser_parsestr_roundtrips_headers_and_body OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/payload_empty_list_makes_container.py`.
#[test]
fn test_gen_behavior_std_libs_email_payload_empty_list_makes_container() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_empty_list_makes_container"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_payload([]) makes the message an (empty) container: get_payload() == []"""
from email.message import Message

container = Message()
container.set_payload([])
assert container.get_payload() == [], container.get_payload()

print("payload_empty_list_makes_container OK")
"###);
    assert_output(&out, r###"payload_empty_list_makes_container OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/payload_set_with_charset_records_input_charset.py`.
#[test]
fn test_gen_behavior_std_libs_email_payload_set_with_charset_records_input_charset() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_set_with_charset_records_input_charset"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_payload(text, Charset('iso-8859-1')) records the input charset on the message (get_charset().input_charset == 'iso-8859-1')"""
from email.message import Message

from email.charset import Charset

ch = Message()
ch.set_payload("This is a string payload", Charset("iso-8859-1"))
assert ch.get_charset().input_charset == "iso-8859-1", ch.get_charset().input_charset

print("payload_set_with_charset_records_input_charset OK")
"###);
    assert_output(&out, r###"payload_set_with_charset_records_input_charset OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_from_mangling__test_multipart_with_bad_bytes_in_cte.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_from_mangling__test_multipart_with_bad_bytes_in_cte() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_from_mangling__test_multipart_with_bad_bytes_in_cte"
# subject = "cpython.test_email.TestFromMangling.test_multipart_with_bad_bytes_in_cte"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser
self_msg = Message()
self_msg['From'] = 'aaa@bbb.org'
self_msg.set_payload('From the desk of A.A.A.:\nBlah blah blah\n')
source = textwrap.dedent('            From: aperson@example.com\n            Content-Type: multipart/mixed; boundary="1"\n            Content-Transfer-Encoding: È\n        ').encode('utf-8')
msg = email.message_from_bytes(source)

print("TestFromMangling::test_multipart_with_bad_bytes_in_cte: ok")
"###);
    assert_output(&out, r###"TestFromMangling::test_multipart_with_bad_bytes_in_cte: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_m_i_m_e_text__test_payload.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_m_i_m_e_text__test_payload() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_m_i_m_e_text__test_payload"
# subject = "cpython.test_email.TestMIMEText.test_payload"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser
self__msg = MIMEText('hello there')
assert self__msg.get_payload() == 'hello there'
assert not self__msg.is_multipart()

print("TestMIMEText::test_payload: ok")
"###);
    assert_output(&out, r###"TestMIMEText::test_payload: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_decode_multiple_spaces.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_decode_multiple_spaces() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_decode_multiple_spaces"
# subject = "cpython.test_email.TestQuopri.test_decode_multiple_spaces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_decode(' ' * 5, '')

print("TestQuopri::test_decode_multiple_spaces: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_decode_multiple_spaces: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_decode_null_word.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_decode_null_word() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_decode_null_word"
# subject = "cpython.test_email.TestQuopri.test_decode_null_word"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_decode('', '')

print("TestQuopri::test_decode_null_word: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_decode_null_word: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_decode_one_space.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_decode_one_space() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_decode_one_space"
# subject = "cpython.test_email.TestQuopri.test_decode_one_space"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_decode(' ', '')

print("TestQuopri::test_decode_one_space: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_decode_one_space: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_encode_null.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_encode_null() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_encode_null"
# subject = "cpython.test_email.TestQuopri.test_encode_null"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_encode('', '')

print("TestQuopri::test_encode_null: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_encode_null: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_header_decode_null.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_header_decode_null() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_header_decode_null"
# subject = "cpython.test_email.TestQuopri.test_header_decode_null"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_header_decode('', '')

print("TestQuopri::test_header_decode_null: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_header_decode_null: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/test_quopri__test_header_encode_null.py`.
#[test]
fn test_gen_behavior_std_libs_email_test_quopri__test_header_encode_null() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "test_quopri__test_header_encode_null"
# subject = "cpython.test_email.TestQuopri.test_header_encode_null"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email/test_email.py"
# status = "filled"
# ///
import re
import time
import base64
import textwrap
from io import StringIO, BytesIO
from itertools import chain
from random import choice
from threading import Thread
import email
import email.policy
import email.utils
from email.charset import Charset
from email.generator import Generator, DecodedGenerator, BytesGenerator
from email.header import Header, decode_header, make_header
from email.headerregistry import HeaderRegistry
from email.message import Message
from email.mime.application import MIMEApplication
from email.mime.audio import MIMEAudio
from email.mime.base import MIMEBase
from email.mime.image import MIMEImage
from email.mime.message import MIMEMessage
from email.mime.multipart import MIMEMultipart
from email.mime.nonmultipart import MIMENonMultipart
from email.mime.text import MIMEText
from email.parser import Parser, HeaderParser
from email import base64mime
from email import encoders
from email import errors
from email import iterators
from email import quoprimime
from email import utils
from email.parser import FeedParser

def _test_header_encode(header, expected_encoded_header, charset=None):
    if charset is None:
        encoded_header = quoprimime.header_encode(header)
    else:
        encoded_header = quoprimime.header_encode(header, charset)
    assert encoded_header == expected_encoded_header

def _test_header_decode(encoded_header, expected_decoded_header):
    decoded_header = quoprimime.header_decode(encoded_header)
    assert decoded_header == expected_decoded_header

def _test_decode(encoded, expected_decoded, eol=None):
    if eol is None:
        decoded = quoprimime.decode(encoded)
    else:
        decoded = quoprimime.decode(encoded, eol=eol)
    assert decoded == expected_decoded

def _test_encode(body, expected_encoded_body, maxlinelen=None, eol=None):
    kwargs = {}
    if maxlinelen is None:
        maxlinelen = 76
    else:
        kwargs['maxlinelen'] = maxlinelen
    if eol is None:
        eol = '\n'
    else:
        kwargs['eol'] = eol
    encoded_body = quoprimime.body_encode(body, **kwargs)
    assert encoded_body == expected_encoded_body
    if eol == '\n' or eol == '\r\n':
        for line in encoded_body.splitlines():
            assert len(line) <= maxlinelen
self_hlit = list(chain(range(ord('a'), ord('z') + 1), range(ord('A'), ord('Z') + 1), range(ord('0'), ord('9') + 1), (c for c in b'!*+-/')))
self_hnon = [c for c in range(256) if c not in self_hlit]
assert len(self_hlit) + len(self_hnon) == 256
self_blit = list(range(ord(' '), ord('~') + 1))
self_blit.append(ord('\t'))
self_blit.remove(ord('='))
self_bnon = [c for c in range(256) if c not in self_blit]
assert len(self_blit) + len(self_bnon) == 256
_test_header_encode(b'', '')

print("TestQuopri::test_header_encode_null: ok")
"###);
    assert_output(&out, r###"TestQuopri::test_header_encode_null: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/utils_formataddr_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_email_utils_formataddr_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "utils_formataddr_roundtrip"
# subject = "email.utils.formataddr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.utils.formataddr: formataddr((name, addr)) emits a display string carrying both the name and the address"""
import email.utils

formatted = email.utils.formataddr(("Alice Smith", "alice@example.com"))
assert "Alice Smith" in formatted, f"formataddr name = {formatted!r}"
assert "alice@example.com" in formatted, f"formataddr addr = {formatted!r}"

print("utils_formataddr_roundtrip OK")
"###);
    assert_output(&out, r###"utils_formataddr_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/email/utils_parseaddr_formats.py`.
#[test]
fn test_gen_behavior_std_libs_email_utils_parseaddr_formats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "utils_parseaddr_formats"
# subject = "email.utils.parseaddr"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.utils.parseaddr: parseaddr splits several address spellings (bare, named-angle, quoted-name) into the expected (realname, addr) pairs"""
import email.utils

cases = [
    ("Alice <alice@example.com>", ("Alice", "alice@example.com")),
    ("bob@example.com", ("", "bob@example.com")),
    ('"Carol Smith" <carol@example.com>', ("Carol Smith", "carol@example.com")),
]
for src, (exp_name, exp_addr) in cases:
    name, addr = email.utils.parseaddr(src)
    assert addr == exp_addr, f"parseaddr addr for {src!r}: {addr!r}"
    assert name == exp_name, f"parseaddr name for {src!r}: {name!r}"

print("utils_parseaddr_formats OK")
"###);
    assert_output(&out, r###"utils_parseaddr_formats OK
"###);
}
