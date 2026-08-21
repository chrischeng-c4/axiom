# test_email.py — #3419 axis-1 stdlib email AssertionPass seed.
#
# Mamba-authored seed exercising the `email` package surface called out
# in the issue:
#   message_from_string, EmailMessage set_content, get_body, get_payload,
#   multipart parse.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. message_from_string parses a simple message — headers + payload.
#   3. EmailMessage.set_content + get_content + get_content_type.
#   4. Multipart parse — message_from_string on a multipart/mixed
#      envelope yields is_multipart() True + walk() over parts.
#   5. get_payload(i) accesses the i-th part of a multipart message.
#   6. get_body / iter_attachments on a multipart message with the
#      modern policy.
#   7. EmailMessage policy + as_string round-trip.
#
# Boxed-int dodge applied to count/length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: email N asserts` to stdout.

import email
import email.message
import email.policy

_ledger: list[int] = []

# 1. Module identity + public surface.
assert email.__name__ == "email", "email.__name__"
_ledger.append(1)
assert hasattr(email, "message_from_string"), "exposes message_from_string"
_ledger.append(1)
assert hasattr(email.message, "EmailMessage"), "email.message exposes EmailMessage"
_ledger.append(1)
assert hasattr(email.message, "Message"), "email.message exposes Message"
_ledger.append(1)
assert hasattr(email.policy, "default"), "email.policy exposes 'default'"
_ledger.append(1)

# 2. message_from_string — headers + simple text payload.
_simple_src = (
    "From: alice@example.com\r\n"
    "To: bob@example.com\r\n"
    "Subject: Hello, mamba\r\n"
    "Content-Type: text/plain; charset=utf-8\r\n"
    "\r\n"
    "Body line one.\r\n"
    "Body line two.\r\n"
)
_msg = email.message_from_string(_simple_src)
assert _msg["From"] == "alice@example.com", "From header parsed"
_ledger.append(1)
assert _msg["To"] == "bob@example.com", "To header parsed"
_ledger.append(1)
assert _msg["Subject"] == "Hello, mamba", "Subject header parsed"
_ledger.append(1)
assert _msg.get_content_type() == "text/plain", "Content-Type parsed"
_ledger.append(1)
assert _msg.is_multipart() == False, "single-part is_multipart() is False"
_ledger.append(1)
# Payload contains both body lines.
_payload = _msg.get_payload()
assert isinstance(_payload, str), "single-part get_payload() returns str"
_ledger.append(1)
assert "Body line one." in _payload, "payload contains first body line"
_ledger.append(1)
assert "Body line two." in _payload, "payload contains second body line"
_ledger.append(1)

# 3. EmailMessage.set_content + get_content + get_content_type.
_em = email.message.EmailMessage()
_em["From"] = "from@example.com"
_em["To"] = "to@example.com"
_em["Subject"] = "set_content roundtrip"
_em.set_content("set_content payload\n")
assert _em.get_content_type() == "text/plain", (
    "set_content(str) defaults to text/plain"
)
_ledger.append(1)
assert _em.get_content() == "set_content payload\n", (
    "get_content returns set_content payload"
)
_ledger.append(1)

# 4. message_from_string on a multipart/mixed envelope.
_mp_src = (
    "From: alice@example.com\r\n"
    "To: bob@example.com\r\n"
    "Subject: multipart\r\n"
    'Content-Type: multipart/mixed; boundary="BOUND"\r\n'
    "\r\n"
    "--BOUND\r\n"
    "Content-Type: text/plain; charset=utf-8\r\n"
    "\r\n"
    "part one (text)\r\n"
    "--BOUND\r\n"
    "Content-Type: text/html; charset=utf-8\r\n"
    "\r\n"
    "<p>part two (html)</p>\r\n"
    "--BOUND--\r\n"
)
_mp = email.message_from_string(_mp_src, policy=email.policy.default)
assert _mp.is_multipart() == True, "multipart/mixed envelope is_multipart() is True"
_ledger.append(1)
assert _mp.get_content_type() == "multipart/mixed", "Content-Type is multipart/mixed"
_ledger.append(1)
# walk() iterates the envelope + each leaf part.
_walked = list(_mp.walk())
# envelope + 2 parts == 3 visits.
assert len(_walked) - 3 == 0, "walk() visits envelope + 2 parts == 3"
_ledger.append(1)
assert _walked[0] is _mp, "walk()[0] is the envelope itself"
_ledger.append(1)

# 5. get_payload(i) — i-th part access on the legacy API.
_mp_legacy = email.message_from_string(_mp_src)
assert _mp_legacy.is_multipart() == True, "legacy parse also sees multipart"
_ledger.append(1)
_parts = _mp_legacy.get_payload()
assert isinstance(_parts, list), "multipart legacy payload is a list"
_ledger.append(1)
assert len(_parts) - 2 == 0, "two parts in the multipart payload"
_ledger.append(1)
_p0 = _mp_legacy.get_payload(0)
assert _p0.get_content_type() == "text/plain", "part 0 is text/plain"
_ledger.append(1)
_p1 = _mp_legacy.get_payload(1)
assert _p1.get_content_type() == "text/html", "part 1 is text/html"
_ledger.append(1)
# Bodies preserved per-part.
assert "part one (text)" in _p0.get_payload(), "part 0 carries 'part one (text)'"
_ledger.append(1)
assert "part two (html)" in _p1.get_payload(), "part 1 carries 'part two (html)'"
_ledger.append(1)

# 6. get_body — modern policy returns a body part by content type.
_body = _mp.get_body(preferencelist=("html", "plain"))
assert _body is not None, "get_body returns a part for html/plain preference"
_ledger.append(1)
assert _body.get_content_type() == "text/html", (
    "get_body picks text/html when html preferred"
)
_ledger.append(1)
_body_plain = _mp.get_body(preferencelist=("plain",))
assert _body_plain is not None, "get_body returns the plain part when preferred"
_ledger.append(1)
assert _body_plain.get_content_type() == "text/plain", (
    "get_body picks text/plain when only plain preferred"
)
_ledger.append(1)

# 7. EmailMessage policy + as_string round-trip.
_dumped = _em.as_string()
assert isinstance(_dumped, str), "as_string returns str"
_ledger.append(1)
assert "Subject: set_content roundtrip" in _dumped, (
    "as_string output carries the Subject"
)
_ledger.append(1)
assert "set_content payload" in _dumped, (
    "as_string output carries the body"
)
_ledger.append(1)
# Parse the dumped representation back and confirm header + body survived.
_round = email.message_from_string(_dumped)
assert _round["Subject"] == "set_content roundtrip", "header survives round-trip"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: email {len(_ledger)} asserts")
