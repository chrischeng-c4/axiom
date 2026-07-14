# #220 — list() kwargs rejection + container-arg relaxation for ==-search methods

Status: landed (`dde0a6e98` fix-pack). Backfill TD.

## Mechanism

1. `list(sequence=[])` silently succeeded — CPython raises TypeError (list
   takes no keyword arguments). One-line gap: the keyword-rejection arm in
   ast_to_hir matched only `set|frozenset`.
2. `host.index(host)` (self-referential search) hard-aborted compilation:
   the element-typed signatures synthesized for `.index/.count/.remove` exist
   to wall genuinely wrong-typed SCALARS, but CPython's ==-based search never
   raises on shape mismatch — a container-shaped value must not wall.

## Invariant

Keyword-argument semantics are PER-BUILTIN facts: `list`/`set`/`frozenset`
reject kwargs; `dict` ACCEPTS them (#1549 open — do not copy the rejection
pattern there). The scalar walls (`list__index__value_as__T_wrong` guard set)
must survive any relaxation.

## Fix pattern

ast_to_hir: add `"list"` to the kwargs-rejection arm → catchable
`mb_arg_bind_error` TypeError. check_expr: `container_receiver_relaxed_call`
— relax ONLY when receiver is provably builtin List/Set AND the method is
`index|count|remove` AND param_idx==0's actual is a bare container
(List/Set/Dict/Tuple). Re-derive receiver type additively (the
`check_dict_operator_call` idiom).

## Verification contract

`_regression/builtin-libs/list_methods/{errors,reentrancy}.py` byte-identical;
guard fixtures still rejected. Sibling open: #1536 (append-shape reflexive
false positive), #1547 (mixed-type compare operand order).
