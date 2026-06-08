"""Hot-loop bench for `xml.etree.ElementTree.Element` /
`xml.etree.ElementTree.SubElement` /
`xml.etree.ElementTree.parse` /
`xml.etree.ElementTree.tostring` /
`xml.etree.ElementTree.fromstring` /
`xml.etree.ElementTree.ElementTree` /
`xml.etree.ElementTree.QName` /
`xml.etree.ElementTree.ParseError` module-attribute reads (#1481).

End-user scenario: XML serializer / config loader / RSS parser code
typically reads the `xml.etree.ElementTree` constructors and helper
functions on every parse / serialize site rather than caching a
local alias. Wrapper code that branches on
`isinstance(elem, ET.Element)`, builds documents via
`ET.SubElement(parent, tag)`, or short-circuits on
`except ET.ParseError` re-resolves these names through the module's
attribute table on each call site. That per-call module-attribute
octet-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `xml.etree.ElementTree` family are top-level module-dict
probes on 3.12 returning class objects / functions). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `xml.etree.ElementTree` module-attribute
resolver, short-circuiting CPython's module-dict probe chain for
read-only class sentinels.

Workload: 10_000 paired reads of `Element`, `SubElement`, `parse`,
`tostring`, `fromstring`, `ElementTree`, `QName`, and `ParseError`
per iteration, compared by identity (`is`) against the hoisted
baseline references taken once before the loop. The accumulator
increments when all eight reads resolve to identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import xml.etree.ElementTree as _ET

_ELEMENT_BASELINE = _ET.Element
_SUBELEMENT_BASELINE = _ET.SubElement
_PARSE_BASELINE = _ET.parse
_TOSTRING_BASELINE = _ET.tostring
_FROMSTRING_BASELINE = _ET.fromstring
_ELEMENTTREE_BASELINE = _ET.ElementTree
_QNAME_BASELINE = _ET.QName
_PARSEERROR_BASELINE = _ET.ParseError

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    a = _ET.Element
    b = _ET.SubElement
    c = _ET.parse
    d = _ET.tostring
    e = _ET.fromstring
    f = _ET.ElementTree
    g = _ET.QName
    h = _ET.ParseError
    if (a is _ELEMENT_BASELINE
            and b is _SUBELEMENT_BASELINE
            and c is _PARSE_BASELINE
            and d is _TOSTRING_BASELINE
            and e is _FROMSTRING_BASELINE
            and f is _ELEMENTTREE_BASELINE
            and g is _QNAME_BASELINE
            and h is _PARSEERROR_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"xml.etree.ElementTree module-attribute read acc drift: acc={acc} expected={ITERS}"
print("xml_etree_type_read_hot:", acc)
