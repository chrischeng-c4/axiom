# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generators"
# dimension = "behavior"
# case = "method_dispatch_in_generator_body"
# subject = "generator method dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""generator method dispatch: generator bodies can call bound/native methods."""


def direct_method_call():
    for c in "abc":
        yield c.upper()


def stored_bound_method_call():
    for c in "abc":
        method = c.upper
        yield method()


assert list(direct_method_call()) == ["A", "B", "C"]
assert list(stored_bound_method_call()) == ["A", "B", "C"]
assert list(c.upper() for c in "abc") == ["A", "B", "C"]

print("method_dispatch_in_generator_body OK")
