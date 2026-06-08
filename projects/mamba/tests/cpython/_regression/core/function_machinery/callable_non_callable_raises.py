# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# mamba-xfail: `callable(1)` returns True on mamba; CPython returns
# False. The fixture asserts the CPython contract; mamba diverges.
# See project_mamba_function_machinery_silent_divergences (#6).
"""builtin callable() contract (CPython 3.12 oracle)."""


# CPython: False; mamba: True.
print("int:", callable(1))
print("str:", callable("abc"))
print("list:", callable([1, 2]))
print("dict:", callable({}))
print("none:", callable(None))

# CPython: True for actual callables.
print("fn:", callable(print))
print("type:", callable(int))


# Calling a non-callable raises TypeError on both runtimes; but mamba's
# erroneous True from callable(1) means a guarded call goes through and
# fails for a different reason ("not callable" but with different msg).
try:
    (1)()
    print("call_int: no_raise")
except TypeError as e:
    print(f"call_int: TypeError: {str(e)[:40]}")
