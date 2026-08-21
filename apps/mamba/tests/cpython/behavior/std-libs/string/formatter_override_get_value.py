# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_get_value"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding get_value() resolves names from a namespace dict: NamespaceFormatter({'greeting':'hello'}).format('{greeting}, world!') == 'hello, world!'"""
import string


class NamespaceFormatter(string.Formatter):
    def __init__(self, namespace):
        super().__init__()
        self.namespace = namespace

    def get_value(self, key, args, kwds):
        if isinstance(key, str):
            try:
                return kwds[key]
            except KeyError:
                return self.namespace[key]
        return super().get_value(key, args, kwds)


ns = NamespaceFormatter({"greeting": "hello"})
assert ns.format("{greeting}, world!") == "hello, world!", "get_value override"
print("formatter_override_get_value OK")
