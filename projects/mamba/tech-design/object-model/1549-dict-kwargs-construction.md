# #1549 — dict(**kwargs) and dict-subclass kwargs construction

Status: OPEN (p2). Design for implementation.

## Mechanism

`dict(name='x')` and `D(name='x')` (D a dict subclass) fail with
`TypeError: dict() takes no keyword arguments` — the constructor path never
binds kwargs into entries. CPython semantics: `dict(**kw)` inserts each kwarg
as a str-keyed item; `dict(iterable_or_mapping, **kw)` seeds then updates.

## Invariant

`list`/`set`/`frozenset` REJECT kwargs (per #220, ast_to_hir rejection arm) —
dict is the opposite; do not share code paths that conflate the two
behaviors. Subclass construction must route through the same kwargs binding
(the subclass may lack its own `__init__`).

## Fix direction

Two layers: (1) lowering — ensure dict calls with kwargs pack them (the
method-call auto-pack idiom exists, see tomllib parse_float precedent) instead
of hitting the rejection error; (2) runtime dict constructor — accept the
trailing kwargs dict and insert entries (keys are display strings — use the
Python-semantic DictKey insert path, NOT raw native-hash inserts; see
`1566-dictkey-hash-domain-audit.md`). Check the positional+kwargs combined
form.

## Verification contract

Probe: plain `dict(name='y')`, subclass `D(name='x', value=2)`, combined
`dict([('a',1)], b=2)` — byte-identical vs oracle. Victim fixture:
string_formatting/behavior.py's dict-subclass segment. dict_methods sweep no
regressions; `list(sequence=[])` still raises (the #220 guard).
