# Operational AssertionPass seed for SILENT divergences across the
# xml.etree.ElementTree Element-instance contract + ET.tostring
# attribute/text-content contract + html dotted-import-rebinding
# contract + html.parser HTMLParser identifier + html.entities
# entitydefs / name2codepoint / codepoint2name / html5 identifier
# surface + unicodedata extended module-helper surface (lookup /
# ucd_3_2_0 / combining / east_asian_width / digit / numeric /
# mirrored / decomposition / is_normalized) + unicodedata.name
# full-string value contract pinned by atomic 186:
# `xml.etree.ElementTree` (the documented `type(ET.fromstring(...))
# .__name__ == "Element"` class-identity contract + the documented
# Element.find / .text / .attrib instance method+attribute layer +
# the documented ET.tostring full-bytes attribute/text emission
# contract), `html` (the documented escape / unescape hasattr layer
# AFTER dotted submodule import of html.parser / html.entities),
# `html.parser` (the documented HTMLParser class identifier),
# `html.entities` (the documented entitydefs / name2codepoint /
# codepoint2name / html5 module-level identifier surface), and
# `unicodedata` (the documented extended lookup / ucd_3_2_0 /
# combining / east_asian_width / digit / numeric / mirrored /
# decomposition / is_normalized function / class identifier
# surface + the documented unicodedata.name("A") ==
# "LATIN CAPITAL LETTER A" full-string value contract).
#
# The matching subset (full xml.etree.ElementTree module hasattr
# surface + Element.tag attribute, html.escape / html.unescape
# value (when imported standalone), full codecs module hasattr +
# encode / decode value, partial unicodedata module hasattr (name
# / category / normalize / bidirectional / decimal /
# unidata_version), unicodedata.category("A") == "Lu",
# unicodedata.normalize NFC, full calendar module hasattr +
# MONDAY / SUNDAY integer value) is covered by
# `test_xml_html_codecs_unicodedata_calendar_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(ET.fromstring("<root><child id='1'>text</child></root>"))
#     .__name__ == "Element" — documented class identity
#     (mamba: returns "dict" — the parser short-circuits to a
#     plain dict);
#   • ET.fromstring(...).find("child").tag == "child" —
#     documented instance method+attribute (mamba: raises
#     AttributeError on .find because the instance is a dict);
#   • ET.tostring(root_with_attrs_and_text) ==
#     b'<root><child id="1">hello</child></root>' — documented
#     full-bytes attribute/text emission (mamba: returns
#     '<root><child /></root>' as str — drops attributes,
#     drops text content, returns str not bytes);
#   • hasattr(html, "escape") is True AFTER `import html.parser`
#     / `import html.entities` — documented namespace stability
#     under dotted submodule import (mamba: False — dotted
#     submodule import rebinds the `html` namespace away from
#     the parent module);
#   • hasattr(html.parser, "HTMLParser") is True — documented
#     class identifier (mamba: False);
#   • hasattr(html.entities, "entitydefs") is True — documented
#     module-level dict identifier (mamba: False);
#   • hasattr(html.entities, "name2codepoint") is True —
#     documented module-level dict identifier (mamba: False);
#   • hasattr(html.entities, "codepoint2name") is True —
#     documented module-level dict identifier (mamba: False);
#   • hasattr(html.entities, "html5") is True — documented
#     module-level dict identifier (mamba: False);
#   • hasattr(unicodedata, "lookup") is True — documented
#     function identifier (mamba: False);
#   • hasattr(unicodedata, "ucd_3_2_0") is True — documented
#     sentinel identifier (mamba: False);
#   • hasattr(unicodedata, "combining") is True — documented
#     function identifier (mamba: False);
#   • hasattr(unicodedata, "east_asian_width") is True —
#     documented function identifier (mamba: False);
#   • hasattr(unicodedata, "digit") is True — documented
#     function identifier (mamba: False);
#   • hasattr(unicodedata, "numeric") is True — documented
#     function identifier (mamba: False);
#   • hasattr(unicodedata, "mirrored") is True — documented
#     function identifier (mamba: False);
#   • hasattr(unicodedata, "decomposition") is True —
#     documented function identifier (mamba: False);
#   • hasattr(unicodedata, "is_normalized") is True —
#     documented function identifier (mamba: False);
#   • unicodedata.name("A") == "LATIN CAPITAL LETTER A" —
#     documented full-string value contract (mamba: returns
#     "UNICODE CHAR 0041" — a synthetic placeholder, not the
#     Unicode database name).
import xml.etree.ElementTree as _et_mod
import html as _html_mod
import html.parser as _html_parser_mod
import html.entities as _html_entities_mod
import unicodedata as _unicodedata_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
ET: Any = _et_mod
html: Any = _html_mod
html_parser: Any = _html_parser_mod
html_entities: Any = _html_entities_mod
unicodedata: Any = _unicodedata_mod


_ledger: list[int] = []

# 1) ET.fromstring — Element class identity contract
_e = ET.fromstring("<root><child id='1'>text</child></root>")
assert type(_e).__name__ == "Element"; _ledger.append(1)

# 2) ET.Element.find / .tag — instance method+attribute contract
_c = _e.find("child")
assert _c is not None; _ledger.append(1)
assert _c.tag == "child"; _ledger.append(1)
assert _c.text == "text"; _ledger.append(1)
assert _c.attrib == {"id": "1"}; _ledger.append(1)

# 3) ET.tostring — full-bytes attribute/text emission contract
_root = ET.Element("root")
_child = ET.SubElement(_root, "child", id="1")
_child.text = "hello"
_s = ET.tostring(_root)
assert _s == b'<root><child id="1">hello</child></root>'; _ledger.append(1)

# 4) html — escape / unescape hasattr layer AFTER dotted submodule
#    import of html.parser / html.entities (mamba: False — dotted
#    submodule import rebinds the `html` namespace)
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)

# 5) html.parser — HTMLParser class identifier
assert hasattr(html_parser, "HTMLParser") == True; _ledger.append(1)

# 6) html.entities — module-level identifier surface
assert hasattr(html_entities, "entitydefs") == True; _ledger.append(1)
assert hasattr(html_entities, "name2codepoint") == True; _ledger.append(1)
assert hasattr(html_entities, "codepoint2name") == True; _ledger.append(1)
assert hasattr(html_entities, "html5") == True; _ledger.append(1)

# 7) unicodedata — extended module-helper surface
assert hasattr(unicodedata, "lookup") == True; _ledger.append(1)
assert hasattr(unicodedata, "ucd_3_2_0") == True; _ledger.append(1)
assert hasattr(unicodedata, "combining") == True; _ledger.append(1)
assert hasattr(unicodedata, "east_asian_width") == True; _ledger.append(1)
assert hasattr(unicodedata, "digit") == True; _ledger.append(1)
assert hasattr(unicodedata, "numeric") == True; _ledger.append(1)
assert hasattr(unicodedata, "mirrored") == True; _ledger.append(1)
assert hasattr(unicodedata, "decomposition") == True; _ledger.append(1)
assert hasattr(unicodedata, "is_normalized") == True; _ledger.append(1)

# 8) unicodedata.name("A") — full-string value contract
assert unicodedata.name("A") == "LATIN CAPITAL LETTER A"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_xml_html_unicodedata_silent {sum(_ledger)} asserts")
