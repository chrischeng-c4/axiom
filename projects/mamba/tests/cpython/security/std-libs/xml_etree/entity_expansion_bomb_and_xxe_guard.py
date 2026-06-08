# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "security"
# case = "entity_expansion_bomb_and_xxe_guard"
# subject = "xml.etree.ElementTree.fromstring"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.fromstring: a billion-laughs nested-entity bomb is guarded (ParseError or bounded expansion, never exponential) and a SYSTEM external-entity (XXE) pointing at a real temp file never reads that file's secret contents"""
import os
import tempfile
import xml.etree.ElementTree as ET


# --- (1) Billion-laughs: deeply nested general-entity definitions ----------
# ent0 = "lol"; each ent{n} references ent{n-1} ten times. Fully expanded this
# is 10**11 copies of "lol" (~300 GB). expat's input-amplification guard must
# refuse this -- ElementTree.fromstring RAISES rather than expanding.
_defs = ['<!ENTITY ent0 "lol">']
for _i in range(1, 12):
    _refs = "".join("&ent{};".format(_i - 1) for _ in range(10))
    _defs.append('<!ENTITY ent{} "{}">'.format(_i, _refs))
_doctype = "\n ".join(_defs)
bomb = (
    '<?xml version="1.0"?>\n'
    "<!DOCTYPE lolz [\n {}\n]>\n"
    "<lolz>&ent11;</lolz>"
).format(_doctype)

bomb_guarded = False
try:
    root = ET.fromstring(bomb)
    # If it did not raise, the only safe outcome is a bounded result -- a real
    # exponential expansion would already have exhausted memory. Treat any
    # silent multi-megabyte expansion as a guard failure.
    expanded = len(root.text or "")
    bomb_guarded = expanded < 1_000_000
except ET.ParseError:
    bomb_guarded = True

assert bomb_guarded, "billion-laughs bomb must be guarded, not exponentially expanded"


# --- (2) XXE: SYSTEM external entity pointing at a real local file ---------
# expat does not resolve external general entities by default, so the file is
# never opened and the entity stays undefined -> ParseError. The secret marker
# must NEVER appear in any parsed text.
SECRET = "XXE_SECRET_MARKER_DO_NOT_READ"
with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
    f.write(SECRET)
    secret_path = f.name

try:
    xxe = (
        '<?xml version="1.0"?>\n'
        '<!DOCTYPE foo [ <!ENTITY xxe SYSTEM "file://{}"> ]>\n'
        "<foo>&xxe;</foo>"
    ).format(secret_path)

    file_read = False
    try:
        root = ET.fromstring(xxe)
        text = root.text or ""
        file_read = SECRET in text
    except ET.ParseError:
        # Entity left unexpanded / undefined -> no inclusion happened.
        file_read = False

    assert not file_read, "XXE must not read the local file"
finally:
    os.unlink(secret_path)


print("entity_expansion_bomb_and_xxe_guard OK")
