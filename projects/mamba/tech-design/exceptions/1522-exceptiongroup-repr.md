# #1522 — BaseExceptionGroup repr drops contained exception class names

Status: OPEN (p2). Design for implementation.

## Mechanism

`repr(BaseExceptionGroup("msg", [KeyboardInterrupt(2)]))` renders the
contained exception as bare `2` instead of `KeyboardInterrupt(2)` — the
group's repr path stringifies each contained exception's ARGS instead of
rendering `ClassName(args...)`. Found in #227's regression sweep
(`_regression/core/exception_group/str_repr.py`), pre-existing.

## Invariant

CPython: a group's repr renders each contained exception via its own repr
(`ClassName(arg_repr, ...)`); str(group) = `message (N sub-exception(s))`.
Nested groups render recursively.

## Fix direction

In the ExceptionGroup repr arm (exception.rs / wherever mb_repr dispatches
group instances): map contained values through the standard exception repr
helper rather than arg extraction. Check nested-group and multi-exception
cases while there.

## Verification contract

`_regression/core/exception_group/str_repr.py` byte-identical vs oracle;
`_regression/core/exception_group/` dir sweep no regressions (#227's other
cited fixtures stay green); gate no worse.
