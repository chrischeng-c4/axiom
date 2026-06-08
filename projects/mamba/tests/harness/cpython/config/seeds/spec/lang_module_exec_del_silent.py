# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of exception `__traceback__` exposure
# (the documented "an active or just-caught exception carries
# __traceback__ pointing to its traceback object" — mamba returns
# None, type 'NoneType'), the `__builtins__` namespace surface (the
# documented "dir(__builtins__) lists 150+ builtin names including
# 'print' and 'len'" — mamba has near-empty __builtins__ that does
# not advertise print/len), `exec(src, ns)` namespace assignment
# (the documented "exec writes top-level bindings into the supplied
# globals/locals dict" — mamba leaves the dict-side binding as None),
# `exec(compile(src, ...), ns)` round trip (the documented "compile
# then exec installs bindings into the namespace" — mamba returns
# None for the installed binding), `del lst[a:b]` slice deletion
# (the documented "del list[a:b] mutates the list to drop that slice"
# — mamba leaves the list unchanged), `except VarName as e` lifetime
# (the documented "Python unbinds the as-target after the except
# block" — mamba keeps e bound), `eval(src, globals_dict)`
# globals-dict honor (the documented "eval reads names from the
# globals dict argument" — mamba returns None for any name lookup
# inside the dict), and `__import__('os')` return type (the
# documented "__import__ returns a module object" — mamba returns a
# dict).
# Ten-pack pinned to atomic 326.
#
# Behavioral edges that CONFORM on mamba (exception hierarchy depth
# IndexError/KeyError/ZeroDivisionError/FloatingPointError/
# OverflowError/UnicodeError/UnicodeDecode-Encode/FileNotFound/
# Permission/IsADirectory/NotADirectory/BlockingIO/ChildProcess/
# ConnectionReset-Aborted-Refused/BrokenPipe/ConnectionError/
# StopIteration/StopAsync/GeneratorExit/KeyboardInterrupt/SystemExit
# /Exception/TypeError/NameError/AttributeError/ImportError/
# ModuleNotFound/RuntimeError/Recursion/NotImplemented under correct
# bases, try/except specificity (subclass caught by base except),
# except tuple, finally/else timing (else only when no raise), e.args
# /str/type/repr, raise...from None __cause__/__suppress_context__,
# implicit __context__, custom Exception subclass with code/__str__
# override, StopIteration.value, AssertionError with message,
# BaseException catch, raise instance identity, isinstance Exception
# hierarchy) are covered in the matching pass fixture
# `test_lang_exception_hierarchy_handling_value_ops`.


_ledger: list[int] = []

# 1) e.__traceback__ is not None after raise/catch
#    (mamba: e.__traceback__ is None)
try:
    raise ValueError("x")
except ValueError as e:
    assert e.__traceback__ is not None; _ledger.append(1)

# 2) type(e.__traceback__).__name__ == "traceback"
#    (mamba: 'NoneType')
try:
    raise ValueError("x")
except ValueError as e:
    assert type(e.__traceback__).__name__ == "traceback"; _ledger.append(1)

# 3) __builtins__ exposes 'print' attribute
#    (mamba: __builtins__ does not advertise print)
assert hasattr(__builtins__, "print") == True; _ledger.append(1)

# 4) __builtins__ exposes 'len' attribute
#    (mamba: __builtins__ does not advertise len)
assert hasattr(__builtins__, "len") == True; _ledger.append(1)

# 5) dir(__builtins__) has 100+ entries
#    (mamba: dir(__builtins__) is near-empty)
assert len(dir(__builtins__)) > 100; _ledger.append(1)

# 6) exec(src, ns) writes the binding into ns
#    (mamba: ns[name] becomes None)
_ns = {}
exec("x = 42", _ns)
assert _ns["x"] == 42; _ledger.append(1)

# 7) exec(compile(src, ...), ns) writes the binding into ns
#    (mamba: ns[name] becomes None)
_ns2 = {}
_code = compile("y = 99", "<test>", "exec")
exec(_code, _ns2)
assert _ns2["y"] == 99; _ledger.append(1)

# 8) del lst[a:b] removes the slice
#    (mamba: list unchanged)
_lst = [1, 2, 3, 4, 5]
del _lst[1:4]
assert _lst == [1, 5]; _ledger.append(1)

# 9) eval(src, globals_dict) looks up names from the dict
#    (mamba: returns None)
assert eval("z * 2", {"z": 10}) == 20; _ledger.append(1)

# 10) __import__('os') returns a module object (not a dict)
#     (mamba: returns 'dict')
assert type(__import__("os")).__name__ == "module"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_module_exec_del_silent {sum(_ledger)} asserts")
