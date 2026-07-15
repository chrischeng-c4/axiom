# #1629 — xml_etree children-list/SubElement mechanism

Status: OPEN (p2). Design for implementation. (stdlib/ context; the sibling
attrib-dict bug was `b789a7900` — DIFFERENT mechanism, see
`object-model/1566-dictkey-hash-domain-audit.md`.)

## Mechanism

Several xml_etree fixtures fail on child-element behavior: iteration/len over
an Element's children, `SubElement` attachment, `find/findall` traversal
inconsistencies (#1627's sweep isolated these as NOT attrib-related; one
tostring tag-lookup flake also recorded there). The children storage in
`xml_mod.rs` (element child list) diverges from CPython's list-like Element
semantics — diagnose whether children are stored but not surfaced (accessor
gap) or dropped at parse/attach time.

## Invariant

Element behaves list-like over its children: `len(el)`, `el[i]`, iteration,
`list(el)`; `SubElement(parent, tag)` attaches AND returns the child;
`remove/insert/append` mutate the same backing sequence the iterators see.

## Fix direction

Audit `xml_mod.rs` child-list read paths against the invariant (same
probe-shape as the attrib fix: construct → mutate → read back). If any child
lookup probes a Python-semantic dict with raw `&str`, apply the
`dict_get_exact_str` pattern.

## Verification contract

The failing xml_etree fixtures from #1627's sweep list byte-identical vs
oracle; `behavior/std-libs/xml_etree*` dir sweep — report before/after counts;
the attrib fixtures (keys_lists_attrs_get) stay green; gate no worse.
