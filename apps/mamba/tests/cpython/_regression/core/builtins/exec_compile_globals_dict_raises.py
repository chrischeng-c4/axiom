# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `exec(compile(...), ns)` also doesn't materialize
# bindings into ns on mamba. See
# project_mamba_module_exec_del_silent_divergences (#4).
"""exec(compile(src), ns) namespace binding (CPython 3.12 oracle)."""

ns: dict = {}
code = compile("y = 7", "<test>", "exec")
# CPython: 'code'; mamba: 'NoneType' (compile broken too) or 'code'.
print("code_type:", type(code).__name__)

exec(code, ns)
# CPython: 7; mamba: None.
print("y:", ns.get("y"))

# CPython: True; mamba: False.
print("written:", "y" in ns)
