"""eval() resolves names from supplied globals and locals dictionaries."""

assert eval("x + 1", {"x": 41}) == 42
assert eval("a + b", {"a": 1, "b": 2}) == 3
assert eval("b", {"b": 2}, {"b": 200}) == 200
assert eval("c", {"b": 2}, {"c": 300}) == 300

missing_raised = False
try:
    eval("missing_name", {})
except NameError:
    missing_raised = True

assert missing_raised

print("eval_namespaces OK")
