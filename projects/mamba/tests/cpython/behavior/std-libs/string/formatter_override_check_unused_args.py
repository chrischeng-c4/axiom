# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "formatter_override_check_unused_args"
# subject = "string.Formatter"
# kind = "semantic"
# xfail = "string.Formatter subclassing relies on the format engine that is a silent dict-stub on mamba (repo-memory stdlib_stub_audit_2026_05_26)"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
"""string.Formatter: overriding check_unused_args() rejects any unconsumed argument: a StrictFormatter accepts a template using all args but raises ValueError when an arg is left unused"""
import string


class StrictFormatter(string.Formatter):
    def check_unused_args(self, used_args, args, kwargs):
        unused = set(kwargs.keys())
        unused.update(range(len(args)))
        for arg in used_args:
            unused.remove(arg)
        if unused:
            raise ValueError("unused arguments")


strict = StrictFormatter()
assert strict.format("{0}{i}{1}", 10, 20, i=100) == "1010020", "all args used"
for args, kwargs in [((10, 20), {}), ((10,), {"i": 100})]:
    _raised = False
    try:
        strict.format("{0}", *args, **kwargs)
    except ValueError:
        _raised = True
    assert _raised, "expected ValueError for unused args"
print("formatter_override_check_unused_args OK")
