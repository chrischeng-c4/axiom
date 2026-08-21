# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `eval(src, ns)` doesn't honor the supplied globals
# dict on mamba; name lookups return None. CPython resolves `z` from
# the dict. See project_mamba_module_exec_del_silent_divergences (#7).
"""eval(src, ns) globals dict (CPython 3.12 oracle)."""

# CPython: 20; mamba: None.
print("z2:", eval("z * 2", {"z": 10}))

# CPython: 'a-b' (concat); mamba: None.
print("concat:", eval("a + '-' + b", {"a": "a", "b": "b"}))

# CPython: 25; mamba: None.
print("pow:", eval("p ** 2", {"p": 5}))

# CPython sees only the dict: NameError on unknown name. mamba: None.
try:
    print("missing:", eval("missing_name", {"only": 1}))
except NameError as e:
    print(f"missing: NameError: {str(e)[:40]}")
