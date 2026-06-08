# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "real_world"
# case = "duck_typed_dispatch_by_abc"
# subject = "collections.abc.Mapping"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: a serializer dispatches on collections.abc membership (Mapping vs non-string Sequence vs scalar) to recursively render mixed nested containers, the canonical isinstance-against-ABC routing pattern"""
import collections.abc as abc


def render(value):
    """Render a nested structure, dispatching by collections.abc membership.

    The canonical real-world pattern: classify each node by ABC rather than by
    concrete type, so any registered/duck-typed mapping or sequence is handled
    uniformly. str is a Sequence, so it is explicitly excluded as a scalar.
    """
    if isinstance(value, abc.Mapping):
        items = ",".join(f"{render(k)}:{render(v)}" for k, v in value.items())
        return "{" + items + "}"
    if isinstance(value, abc.Sequence) and not isinstance(value, (str, bytes)):
        return "[" + ",".join(render(v) for v in value) + "]"
    return repr(value)


# A mixed nested document: dict containing a list of dicts and a scalar.
doc = {"name": "root", "kids": [{"id": 1}, {"id": 2}], "count": 2}
out = render(doc)
assert out == "{'name':'root','kids':[{'id':1},{'id':2}],'count':2}", out

# A bare list routes through the Sequence branch.
assert render([1, [2, 3]]) == "[1,[2,3]]", render([1, [2, 3]])

# A scalar str routes through the fallback (NOT the Sequence branch).
assert render("hi") == "'hi'", render("hi")
print("duck_typed_dispatch_by_abc OK")
