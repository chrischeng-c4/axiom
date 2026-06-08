# lang_init_subclass.py — #3361 axis-1 lang __init_subclass__ +
# __class_getitem__ AssertionPass seed.
#
# Mamba-authored seed exercising the metaclass-light subclass-hook
# surface called out in the issue:
#   * __init_subclass__(cls, **kw) runs on subclass creation
#   * Subclass kwargs forwarded from class statement
#   * __class_getitem__ enables MyClass[int] parameterization
#   * Both hooks composable with metaclass
#
# Contract placement: `spec/` — pins outcome Fail. Mamba runtime gap
# #3505 (__init_subclass__ never invoked; __class_getitem__ receives
# stringified type arg) blocks AssertionPass today. Once #3505 lands
# and this seed flips to AssertionPass on mamba, drift detection
# prompts a `git mv spec/lang_init_subclass.py pass/lang_init_subclass.py`.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. __init_subclass__ fires on subclass creation with no kwargs.
#   2. __init_subclass__ receives forwarded class-statement kwargs.
#   3. __init_subclass__ propagates through a chain of subclasses
#      (grandchild also triggers parent hook).
#   4. __class_getitem__ enables MyClass[int] parameterization — the
#      callable receives the actual type object (not a string).
#   5. __class_getitem__ with a tuple argument (MyClass[int, str])
#      receives the tuple verbatim.
#   6. Both hooks composable with a custom metaclass.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: lang_init_subclass N asserts` to stdout.

_ledger: list[int] = []

# Module-level mutable ledger so the hook can record observations
# from class-body scope.
_observed: list[tuple] = []  # type: ignore[type-arg]


# 1. __init_subclass__ fires on subclass creation without kwargs.
class _Base:
    def __init_subclass__(cls, **kw) -> None:  # type: ignore[no-untyped-def]
        _observed.append(("init_subclass", cls.__name__, kw))
        super().__init_subclass__()


class _Child(_Base):
    pass


# Subclass creation triggered the hook exactly once with empty kwargs.
assert len(_observed) - 1 == 0, (
    "exactly one __init_subclass__ call after _Child creation (boxed-dodge)"
)
_ledger.append(1)
_evt = _observed[0]
assert _evt[0] == "init_subclass", "_observed event tag"
_ledger.append(1)
assert _evt[1] == "_Child", "__init_subclass__ receives cls=_Child"
_ledger.append(1)
assert _evt[2] == {}, "__init_subclass__ receives empty kwargs by default"
_ledger.append(1)


# 2. __init_subclass__ receives forwarded class-statement kwargs.
class _ChildWithKwargs(_Base, tag="alpha", level=3):
    pass


# Second event in the ledger should be for _ChildWithKwargs.
assert len(_observed) - 2 == 0, (
    "two __init_subclass__ calls after _ChildWithKwargs (boxed-dodge)"
)
_ledger.append(1)
_evt2 = _observed[1]
assert _evt2[1] == "_ChildWithKwargs", "second event cls name"
_ledger.append(1)
# Kwargs forwarded verbatim.
assert _evt2[2] == {"tag": "alpha", "level": 3}, (
    "__init_subclass__ forwards class-statement kwargs"
)
_ledger.append(1)


# 3. __init_subclass__ propagates down a subclass chain.
class _Grandchild(_ChildWithKwargs):
    pass


assert len(_observed) - 3 == 0, (
    "three __init_subclass__ calls after _Grandchild (boxed-dodge)"
)
_ledger.append(1)
assert _observed[2][1] == "_Grandchild", "grandchild also fires the hook"
_ledger.append(1)


# 4. __class_getitem__ enables MyClass[int] parameterization.
_getitem_args: list = []


class _Container:
    @classmethod
    def __class_getitem__(cls, item):  # type: ignore[no-untyped-def]
        _getitem_args.append(item)
        return (cls, item)


_alias = _Container[int]
assert _getitem_args == [int], (
    "__class_getitem__ receives the actual type object (not a string)"
)
_ledger.append(1)
# Return value of class subscript is whatever __class_getitem__ returns.
assert _alias == (_Container, int), (
    "_Container[int] returns the (_Container, int) tuple from the hook"
)
_ledger.append(1)


# 5. __class_getitem__ with a tuple argument.
_alias2 = _Container[int, str]
assert _getitem_args[-1] == (int, str), (
    "__class_getitem__ receives the tuple verbatim for multi-arg subscript"
)
_ledger.append(1)
assert _alias2 == (_Container, (int, str)), (
    "tuple subscript returns the (cls, (int, str)) tuple"
)
_ledger.append(1)


# 6. Composability with a custom metaclass.
class _Meta(type):
    """Metaclass that records constructions for inspection."""

    instances: list = []

    def __new__(mcs, name, bases, ns, **kw):  # type: ignore[no-untyped-def]
        cls = super().__new__(mcs, name, bases, ns, **kw)
        mcs.instances.append((name, kw))
        return cls

    def __init__(cls, name, bases, ns, **kw):  # type: ignore[no-untyped-def]
        super().__init__(name, bases, ns, **kw)


_meta_observed: list = []


class _BaseMeta(metaclass=_Meta):
    def __init_subclass__(cls, **kw) -> None:  # type: ignore[no-untyped-def]
        _meta_observed.append(("meta_init_subclass", cls.__name__, kw))


# Subclass of metaclass-using parent — both hooks should fire.
class _ChildMeta(_BaseMeta, tag="beta"):
    pass


# __init_subclass__ on the base saw the subclass:
assert len(_meta_observed) - 1 == 0, (
    "metaclass + __init_subclass__: one hook event (boxed-dodge)"
)
_ledger.append(1)
assert _meta_observed[0][1] == "_ChildMeta", "hook sees _ChildMeta name"
_ledger.append(1)
assert _meta_observed[0][2] == {"tag": "beta"}, (
    "metaclass + __init_subclass__: kwargs forwarded"
)
_ledger.append(1)

# Metaclass also fired and recorded:
_meta_names = [name for (name, _kw) in _Meta.instances]
assert "_BaseMeta" in _meta_names, "metaclass __new__ saw _BaseMeta"
_ledger.append(1)
assert "_ChildMeta" in _meta_names, "metaclass __new__ saw _ChildMeta"
_ledger.append(1)
# Metaclass receives class-statement kwargs too.
_child_meta_kw = [kw for (name, kw) in _Meta.instances if name == "_ChildMeta"][0]
assert _child_meta_kw == {"tag": "beta"}, (
    "metaclass __new__ receives class-statement kwargs alongside __init_subclass__"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: lang_init_subclass {len(_ledger)} asserts")
