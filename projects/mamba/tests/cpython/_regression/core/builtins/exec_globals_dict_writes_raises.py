# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `exec(src, ns)` does not materialize bindings into the
# supplied namespace dict on mamba; CPython writes them in. See
# project_mamba_module_exec_del_silent_divergences (#3).
"""exec(src, ns) namespace binding (CPython 3.12 oracle)."""

ns: dict = {}
exec("x = 42", ns)
# CPython: 42; mamba: None.
print("x:", ns.get("x"))

# CPython: True; mamba: False.
print("written:", "x" in ns)

ns2: dict = {}
exec("y = 'hi'; z = [1, 2, 3]", ns2)
# CPython: 'hi' and [1,2,3]; mamba: None for both.
print("y:", ns2.get("y"))
print("z:", ns2.get("z"))
