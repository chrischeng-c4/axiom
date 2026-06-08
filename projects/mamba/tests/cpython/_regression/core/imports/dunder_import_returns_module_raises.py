# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `__import__('os')` returns the module's __dict__ on
# mamba; CPython returns the module object. See
# project_mamba_module_exec_del_silent_divergences (#8).
"""__import__ return type (CPython 3.12 oracle)."""

os_mod = __import__("os")

# CPython: 'module'; mamba: 'dict'.
print("type:", type(os_mod).__name__)

# CPython: True; mamba: False.
print("is_module_like:", hasattr(os_mod, "getcwd"))

# CPython: 'os'; mamba: TypeError ('dict' has no __name__).
try:
    print("name:", os_mod.__name__)
except AttributeError as e:
    print(f"name: AttributeError: {str(e)[:40]}")

# Same for a stdlib pure-Python module.
sys_mod = __import__("sys")
print("sys_type:", type(sys_mod).__name__)
