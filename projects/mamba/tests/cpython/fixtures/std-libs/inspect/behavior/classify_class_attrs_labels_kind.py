# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "classify_class_attrs_labels_kind"
# subject = "inspect.classify_class_attrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.classify_class_attrs: classify_class_attrs() labels each attribute's kind (data / static method / class method / method) and defining class"""
import inspect

class _Mix:
    data = 5

    @staticmethod
    def st():
        pass

    @classmethod
    def cm(cls):
        pass

    def inst(self):
        pass

_classified = {a.name: a for a in inspect.classify_class_attrs(_Mix)}
assert _classified["data"].kind == "data", f"data kind = {_classified['data'].kind!r}"
assert _classified["st"].kind == "static method", f"st kind = {_classified['st'].kind!r}"
assert _classified["cm"].kind == "class method", f"cm kind = {_classified['cm'].kind!r}"
assert _classified["inst"].kind == "method", f"inst kind = {_classified['inst'].kind!r}"
assert _classified["data"].defining_class is _Mix, "data defined by _Mix"

print("classify_class_attrs_labels_kind OK")
