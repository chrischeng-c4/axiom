# Operational AssertionPass seed for the value contract of the
# `xml.etree.ElementTree` / `html` / `codecs` / `unicodedata` /
# `calendar` five-pack pinned to atomic 186: `xml.etree.ElementTree`
# (the documented full module-level helper hasattr surface —
# `Element` / `ElementTree` / `fromstring` / `tostring` / `parse`
# / `XML` / `SubElement` / `Comment` / `ProcessingInstruction` /
# `QName` / `iselement` + the documented Element.tag attribute
# contract), `html` (the documented `escape` / `unescape` helper
# identifiers + the documented html.escape / html.unescape value
# contract), `codecs` (the documented full module-level helper
# hasattr surface — `encode` / `decode` / `lookup` / `lookup_error`
# / `BOM_UTF8` / `BOM_UTF16` / `BOM_UTF32` / `BOM_UTF16_LE` /
# `BOM_UTF16_BE` / `BOM_UTF32_LE` / `BOM_UTF32_BE` / `register` /
# `getreader` / `getwriter` / `open` + the documented codecs.encode
# / codecs.decode value contract), `unicodedata` (the documented
# partial module-level helper hasattr surface — `name` / `category`
# / `normalize` / `bidirectional` / `decimal` / `unidata_version`
# + the documented unicodedata.category("A") == "Lu" value contract
# + the documented unicodedata.normalize NFC value contract), and
# `calendar` (the documented full module-level helper hasattr
# surface — `Calendar` / `TextCalendar` / `HTMLCalendar` /
# `month_name` / `day_name` / `isleap` / `leapdays` / `monthrange`
# / `weekday` / `timegm` / `EPOCH` / `MONDAY` / `TUESDAY` /
# `WEDNESDAY` / `THURSDAY` / `FRIDAY` / `SATURDAY` / `SUNDAY` +
# the documented calendar.MONDAY / SUNDAY integer value contract).
#
# The matching subset between mamba and CPython is the full
# `xml.etree.ElementTree` module hasattr surface + the Element.tag
# attribute layer (the Element.find / .text / .attrib instance
# layer DIVERGES + the ET.tostring with attribute/text content
# DIVERGES), the `html` escape / unescape value layer (when
# imported standalone — importing `html.parser` / `html.entities`
# alongside breaks the `html` namespace), the full `codecs`
# module hasattr surface + the codecs.encode / codecs.decode
# value layer, the partial `unicodedata` module hasattr surface
# (name / category / normalize / bidirectional / decimal /
# unidata_version — lookup / ucd_3_2_0 / combining /
# east_asian_width / digit / numeric / mirrored / decomposition
# / is_normalized DIVERGE + the unicodedata.name("A") ==
# "LATIN CAPITAL LETTER A" value contract DIVERGES) + the
# category("A") == "Lu" value layer + the normalize NFC value
# layer, and the full `calendar` module hasattr surface + the
# MONDAY / SUNDAY integer value layer.
#
# Surface in this fixture:
#   • xml.etree.ElementTree — full module hasattr surface
#     (Element / ElementTree / fromstring / tostring / parse /
#     XML / SubElement / Comment / ProcessingInstruction /
#     QName / iselement);
#   • xml.etree.ElementTree.Element — tag attribute contract;
#   • html — escape / unescape value contract;
#   • codecs — full module hasattr surface (encode / decode /
#     lookup / lookup_error / BOM_UTF8 / BOM_UTF16 / BOM_UTF32
#     / BOM_UTF16_LE / BOM_UTF16_BE / BOM_UTF32_LE /
#     BOM_UTF32_BE / register / getreader / getwriter / open);
#   • codecs.encode / codecs.decode — value contract;
#   • unicodedata — partial module hasattr surface (name /
#     category / normalize / bidirectional / decimal /
#     unidata_version);
#   • unicodedata.category("A") == "Lu";
#   • unicodedata.normalize("NFC", x) — value contract;
#   • calendar — full module hasattr surface (Calendar /
#     TextCalendar / HTMLCalendar / month_name / day_name /
#     isleap / leapdays / monthrange / weekday / timegm /
#     EPOCH / MONDAY / TUESDAY / WEDNESDAY / THURSDAY /
#     FRIDAY / SATURDAY / SUNDAY);
#   • calendar.MONDAY / SUNDAY — integer value contract.
#
# Behavioral edges that DIVERGE on mamba
# (type(ET.fromstring(...)).__name__ returns "dict" not
# "Element", Element.find / .text / .attrib raise
# AttributeError, ET.tostring drops attributes / text,
# importing html.parser / html.entities rebinds the `html`
# module namespace (hasattr(html, "escape") False after
# dotted submodule import), hasattr(html.parser,
# "HTMLParser") False, hasattr(html.entities, "entitydefs")
# / "name2codepoint" / "codepoint2name" / "html5" all False,
# hasattr(unicodedata, "lookup") / "ucd_3_2_0" / "combining"
# / "east_asian_width" / "digit" / "numeric" / "mirrored" /
# "decomposition" / "is_normalized" all False,
# unicodedata.name("A") returns "UNICODE CHAR 0041" not
# "LATIN CAPITAL LETTER A") are covered in the matching
# spec fixture
# `lang_xml_html_unicodedata_silent`.
import xml.etree.ElementTree as ET
import html
import codecs
import unicodedata
import calendar


_ledger: list[int] = []

# 1) xml.etree.ElementTree — full module hasattr surface
assert hasattr(ET, "Element") == True; _ledger.append(1)
assert hasattr(ET, "ElementTree") == True; _ledger.append(1)
assert hasattr(ET, "fromstring") == True; _ledger.append(1)
assert hasattr(ET, "tostring") == True; _ledger.append(1)
assert hasattr(ET, "parse") == True; _ledger.append(1)
assert hasattr(ET, "XML") == True; _ledger.append(1)
assert hasattr(ET, "SubElement") == True; _ledger.append(1)
assert hasattr(ET, "Comment") == True; _ledger.append(1)
assert hasattr(ET, "ProcessingInstruction") == True; _ledger.append(1)
assert hasattr(ET, "QName") == True; _ledger.append(1)
assert hasattr(ET, "iselement") == True; _ledger.append(1)

# 2) ET.Element — tag attribute contract
_e = ET.fromstring("<root><child id='1'>text</child></root>")
assert _e.tag == "root"; _ledger.append(1)

# 3) html — escape / unescape value contract
assert html.escape("<p>") == "&lt;p&gt;"; _ledger.append(1)
assert html.escape("a & b") == "a &amp; b"; _ledger.append(1)
assert html.unescape("&lt;p&gt;") == "<p>"; _ledger.append(1)

# 4) codecs — full module hasattr surface
assert hasattr(codecs, "encode") == True; _ledger.append(1)
assert hasattr(codecs, "decode") == True; _ledger.append(1)
assert hasattr(codecs, "lookup") == True; _ledger.append(1)
assert hasattr(codecs, "lookup_error") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF8") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF16") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF32") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF16_LE") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF16_BE") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF32_LE") == True; _ledger.append(1)
assert hasattr(codecs, "BOM_UTF32_BE") == True; _ledger.append(1)
assert hasattr(codecs, "register") == True; _ledger.append(1)
assert hasattr(codecs, "getreader") == True; _ledger.append(1)
assert hasattr(codecs, "getwriter") == True; _ledger.append(1)
assert hasattr(codecs, "open") == True; _ledger.append(1)

# 5) codecs.encode / codecs.decode — value contract
assert codecs.encode("hello") == b"hello"; _ledger.append(1)
assert codecs.decode(b"hello") == "hello"; _ledger.append(1)

# 6) unicodedata — partial module hasattr surface
#    (lookup / ucd_3_2_0 / combining / east_asian_width /
#    digit / numeric / mirrored / decomposition /
#    is_normalized DIVERGE — moved to spec fixture)
assert hasattr(unicodedata, "name") == True; _ledger.append(1)
assert hasattr(unicodedata, "category") == True; _ledger.append(1)
assert hasattr(unicodedata, "normalize") == True; _ledger.append(1)
assert hasattr(unicodedata, "bidirectional") == True; _ledger.append(1)
assert hasattr(unicodedata, "decimal") == True; _ledger.append(1)
assert hasattr(unicodedata, "unidata_version") == True; _ledger.append(1)

# 7) unicodedata.category("A") == "Lu" value contract
assert unicodedata.category("A") == "Lu"; _ledger.append(1)

# 8) unicodedata.normalize NFC value contract
assert unicodedata.normalize("NFC", "café") == "café"; _ledger.append(1)

# 9) calendar — full module hasattr surface
assert hasattr(calendar, "Calendar") == True; _ledger.append(1)
assert hasattr(calendar, "TextCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "HTMLCalendar") == True; _ledger.append(1)
assert hasattr(calendar, "month_name") == True; _ledger.append(1)
assert hasattr(calendar, "day_name") == True; _ledger.append(1)
assert hasattr(calendar, "isleap") == True; _ledger.append(1)
assert hasattr(calendar, "leapdays") == True; _ledger.append(1)
assert hasattr(calendar, "monthrange") == True; _ledger.append(1)
assert hasattr(calendar, "weekday") == True; _ledger.append(1)
assert hasattr(calendar, "timegm") == True; _ledger.append(1)
assert hasattr(calendar, "EPOCH") == True; _ledger.append(1)
assert hasattr(calendar, "MONDAY") == True; _ledger.append(1)
assert hasattr(calendar, "TUESDAY") == True; _ledger.append(1)
assert hasattr(calendar, "WEDNESDAY") == True; _ledger.append(1)
assert hasattr(calendar, "THURSDAY") == True; _ledger.append(1)
assert hasattr(calendar, "FRIDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SATURDAY") == True; _ledger.append(1)
assert hasattr(calendar, "SUNDAY") == True; _ledger.append(1)

# 10) calendar.MONDAY / SUNDAY — integer value contract
assert calendar.MONDAY == 0; _ledger.append(1)
assert calendar.SUNDAY == 6; _ledger.append(1)

# NB: type(ET.fromstring(...)).__name__ returns "dict" on
# mamba, Element.find / .text / .attrib raise
# AttributeError, ET.tostring drops attributes/text content,
# importing html.parser / html.entities rebinds the html
# module namespace (hasattr(html, "escape") False after
# dotted submodule import), hasattr(html.parser,
# "HTMLParser") False, hasattr(html.entities, "entitydefs")
# / "name2codepoint" / "codepoint2name" / "html5" all False,
# hasattr(unicodedata, "lookup") / "ucd_3_2_0" / "combining"
# / "east_asian_width" / "digit" / "numeric" / "mirrored" /
# "decomposition" / "is_normalized" all False,
# unicodedata.name("A") returns "UNICODE CHAR 0041" — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_xml_html_codecs_unicodedata_calendar_value_ops {sum(_ledger)} asserts")
