# Operational AssertionPass seed for SILENT divergences across the
# enum-member / email-message / xml-element triplet pinned by
# atomic 170: `enum` (the documented `Enum.MEMBER` str-repr
# `"ClsName.MEMBER"` contract + the documented `.value` /
# `.name` attribute round-trip + the documented `Cls(int)` value
# lookup + the documented `Cls["NAME"]` subscript lookup + the
# documented `len(Cls)` / `list(Cls)` iteration contract),
# `email` (the documented `email.message_from_string` header-
# population contract + the documented `email.message`
# submodule binding on the package), and `xml.etree.ElementTree`
# (the documented `tostring(elem)` bytes-return contract that
# round-trips the element including its `.text`).
#
# The matching subset (urllib.parse from-import path layer +
# urlparse component access + urlencode / quote / unquote /
# urljoin / urlsplit / parse_qs / parse_qsl, http.HTTPStatus
# integer-value + IntEnum arithmetic + equality layer,
# xml.etree.ElementTree module attribute hasattr surface +
# fromstring tag layer, enum.IntEnum arithmetic + equality
# layer) is covered by
# `test_urllib_http_etree_intenum_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • _Color.RED.value == 1 — documented Enum member `.value`
#     attribute (mamba: returns None — `.value` not surfaced);
#   • _Color.RED.name == "RED" — documented Enum member
#     `.name` attribute (mamba: returns None);
#   • _Color(1) is _Color.RED — documented int-value lookup
#     constructor (mamba: returns "Color()" — calls the
#     constructor not the value-to-member lookup);
#   • _Color["RED"] is _Color.RED — documented `__class_getitem__`
#     subscript lookup (mamba: TypeError 'type' object is not
#     subscriptable);
#   • len(_Color) == 3 — documented enum-class length
#     contract (mamba: returns a wrong count);
#   • list(_Color) yields three valid members — documented
#     enum-class iteration contract (mamba: yields None entries
#     so the iteration is unusable);
#   • email.message_from_string("Subject: Hi\nFrom: a@b.com\n
#     \nbody")["Subject"] == "Hi" — documented header-population
#     contract (mamba: KeyError 'Subject' — message is returned
#     but headers are not populated);
#   • xml.etree.ElementTree.tostring(elem) returns bytes
#     b"<foo>bar</foo>" — documented bytes-return contract
#     including element `.text` (mamba: returns str "<foo />"
#     — text is dropped and the return is `str`, not `bytes`).
import email as _email_mod
import xml.etree.ElementTree as _et_mod
from enum import Enum
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# instance methods / class identifiers that mamba's bundled
# type stubs do not surface accurately.
email: Any = _email_mod
ET: Any = _et_mod


class _Color(Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


_ledger: list[int] = []

# 1) enum.Enum — .value / .name attribute round-trip
assert _Color.RED.value == 1; _ledger.append(1)
assert _Color.RED.name == "RED"; _ledger.append(1)
assert _Color.GREEN.value == 2; _ledger.append(1)
assert _Color.BLUE.name == "BLUE"; _ledger.append(1)

# 2) enum.Enum — Cls(int) value lookup
assert _Color(1) is _Color.RED; _ledger.append(1)
assert _Color(2) is _Color.GREEN; _ledger.append(1)

# 3) enum.Enum — Cls["NAME"] subscript lookup
assert _Color["RED"] is _Color.RED; _ledger.append(1)
assert _Color["GREEN"] is _Color.GREEN; _ledger.append(1)

# 4) enum.Enum — len + list iteration
assert len(_Color) == 3; _ledger.append(1)
assert [m.name for m in _Color] == ["RED", "GREEN", "BLUE"]; _ledger.append(1)

# 5) email — message_from_string header population
_msg = email.message_from_string("Subject: Hi\nFrom: a@b.com\n\nbody")
assert _msg["Subject"] == "Hi"; _ledger.append(1)
assert _msg["From"] == "a@b.com"; _ledger.append(1)

# 6) xml.etree.ElementTree — tostring bytes-return with text
_el = ET.Element("foo")
_el.text = "bar"
assert ET.tostring(_el) == b"<foo>bar</foo>"; _ledger.append(1)
assert isinstance(ET.tostring(_el), bytes); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_enum_email_etree_silent {sum(_ledger)} asserts")
