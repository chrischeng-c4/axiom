# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xml_etree"
# dimension = "real_world"
# case = "element_child_mutation_workflow"
# subject = "xml.etree.ElementTree.Element"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_xml_etree.py"
# status = "filled"
# ///
"""xml.etree.ElementTree.Element: an editing workflow drives append/remove/insert (incl. negative index)/extend/clear/makeelement plus list-style slice read, slice assign/delete, single-index assign, and confirms the serialized result reflects the mutations"""
from xml.etree.ElementTree import Element, SubElement, tostring


def tags(elem):
    return [c.tag for c in elem]


# append adds a child at the end; remove deletes it.
root = Element("body")
child = Element("tag")
root.append(child)
assert tags(root) == ["tag"], f"after append = {tags(root)!r}"
root.remove(child)
assert tags(root) == [], f"after remove = {tags(root)!r}"

# insert places a child at an index; negative index inserts before last.
a = Element("a")
SubElement(a, "b")
SubElement(a, "c")
a.insert(0, Element("d"))
assert tags(a) == ["d", "b", "c"], f"after insert 0 = {tags(a)!r}"
a.insert(-1, Element("e"))
assert tags(a) == ["d", "b", "e", "c"], f"after insert -1 = {tags(a)!r}"

# extend appends from any iterable.
e = Element("e")
e.extend([Element("x"), Element("y")])
e.extend(iter([Element("z")]))
assert tags(e) == ["x", "y", "z"], f"after extend = {tags(e)!r}"

# clear wipes children, text, and attributes.
f = Element("f", {"k": "v"})
f.text = "hi"
SubElement(f, "g")
f.clear()
assert tags(f) == [] and f.text is None and f.attrib == {}, \
    f"after clear: tags={tags(f)!r} text={f.text!r} attrib={f.attrib!r}"

# makeelement copies the attrib dict (no aliasing).
parent = Element("tag")
attrib = {"key": "value"}
sub = parent.makeelement("subtag", attrib)
attrib["key"] = "mutated"
assert sub.attrib == {"key": "value"}, f"makeelement aliased attrib = {sub.attrib!r}"

# Slice read: getslice with steps.
s = Element("s")
for i in range(6):
    SubElement(s, "a%d" % i)
assert [c.tag for c in s[3:]] == ["a3", "a4", "a5"], "slice [3:]"
assert [c.tag for c in s[::2]] == ["a0", "a2", "a4"], "slice [::2]"
assert [c.tag for c in s[::-1]] == ["a5", "a4", "a3", "a2", "a1", "a0"], "slice [::-1]"

# Slice assign and slice delete.
s[1:3] = [Element("b0"), Element("b1")]
assert tags(s) == ["a0", "b0", "b1", "a3", "a4", "a5"], f"setslice = {tags(s)!r}"
del s[0:2]
assert tags(s) == ["b1", "a3", "a4", "a5"], f"delslice = {tags(s)!r}"

# Single-index assignment replaces in place.
s[0] = Element("z")
assert tags(s) == ["z", "a3", "a4", "a5"], f"index assign = {tags(s)!r}"

# Serialized result reflects the mutations.
assert tostring(s) == b"<s><z /><a3 /><a4 /><a5 /></s>", f"serialize = {tostring(s)!r}"

print("element_child_mutation_workflow OK")
