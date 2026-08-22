# execution context — ownership inventory rows

Generated appendix to `ownership-inventory.md`. One row per declaration.
Do not hand-edit; regenerate with the scanner named in that document.

Live rows: **349**. Discarded: **26**. Total: **375**.

`mut sites` counts writes found by the mutation scanner; it is a heuristic
upper bound (`.lock()` and `.get_or_init()` register as writes).

Line numbers were captured with uncommitted work in the tree; 52 rows across
five files will shift when it lands. The set is unaffected — see the set digest
in `ownership-inventory.md`.

## Live rows

| symbol | path:line | storage | mut sites | reset path | ownership | destination |
|---|---|---|---|---|---|---|
| `CURRENT_COROUTINE_ID` | `src/runtime/async_rt.rs:318` | thread_local | 5 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `AWAIT_DEADLINE` | `src/runtime/async_task.rs:19` | thread_local | 1 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `CLASSCELL_REQUIRED` | `src/runtime/class/mod.rs:163` | thread_local | 5 | reachable | child-owned | ExecutionChild, joined at quiescence |
| `CLASSCELL_SYMBOL_IDS` | `src/runtime/class/mod.rs:166` | thread_local | 4 | reachable | child-owned | ExecutionChild, joined at quiescence |
| `CLASSCELL_VALUES` | `src/runtime/class/mod.rs:170` | thread_local | 3 | reachable | child-owned | ExecutionChild, joined at quiescence |
| `METACLASS_DEFINITION_STACK` | `src/runtime/class/mod.rs:175` | thread_local | 3 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `SORT_MUTATION_WATCHES` | `src/runtime/list_ops.rs:54` | thread_local | 3 | reachable | child-owned | ExecutionChild, joined at quiescence |
| `NEXT_SORT_MUTATION_GUARD_ID` | `src/runtime/list_ops.rs:55` | thread_local | 1 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `CAPTURE_BUF` | `src/runtime/output.rs:16` | thread_local | 4 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `STDOUT_REDIRECT` | `src/runtime/output.rs:22` | thread_local | 3 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `STDERR_REDIRECT` | `src/runtime/output.rs:25` | thread_local | 2 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `LEAK_BALANCE_STATE` | `src/runtime/rc.rs:1434` | thread_local | 6 | reachable | child-owned | ExecutionChild, joined at quiescence |
| `IN_PROGRESS` | `src/runtime/repr_guard.rs:15` | thread_local | 2 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `DEEPCOPY_ORDER` | `src/runtime/stdlib/copy_mod.rs:423` | thread_local | 2 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `PENDING_VERIFY_CHECKS` | `src/runtime/stdlib/enum_mod.rs:647` | thread_local | 1 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `SAMPLES_NONCE` | `src/runtime/stdlib/statistics_mod.rs:1213` | thread_local | 1 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `RECURSION_DEPTH` | `src/runtime/stdlib/sys_mod.rs:246` | thread_local | 2 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `TRACE_FRAME_STACK` | `src/runtime/stdlib/traceback_mod.rs:71` | thread_local | 9 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `XML_MISMATCH_POS` | `src/runtime/stdlib/xml_mod.rs:1776` | thread_local | 3 | NONE | child-owned | ExecutionChild, joined at quiescence |
| `GIL_HELD` | `src/runtime/async_task.rs:1378` | thread_local | 2 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `LAST_CAUGHT_VALUE` | `src/runtime/class/mod.rs:5305` | thread_local | 4 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `LAST_RAISED_INSTANCE` | `src/runtime/class/mod.rs:5392` | thread_local | 7 | reachable | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `CURRENT_EXCEPTION` | `src/runtime/exception.rs:125` | thread_local | 16 | reachable | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `EXCEPTION_HANDLERS` | `src/runtime/exception.rs:126` | thread_local | 3 | reachable | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `LAST_HANDLED_EXCEPTION` | `src/runtime/exception.rs:132` | thread_local | 3 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `HANDLED_EXC_SAVE_STACK` | `src/runtime/exception.rs:139` | thread_local | 2 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `CURRENT_THREAD_OBJ` | `src/runtime/stdlib/threading_mod.rs:434` | thread_local | 5 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `LOCAL_INSTANCES` | `src/runtime/stdlib/threading_mod.rs:1308` | thread_local | 1 | NONE | compatibility-binding | scoped TLS stack holding ContextHandle only |
| `SHIM_JIT` | `src/runtime/builtins/wide_call.rs:176` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CLASS_REGISTRY` | `src/runtime/class/mod.rs:124` | thread_local | 34 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CLASS_RUNTIME_KEY_ALIASES` | `src/runtime/class/mod.rs:126` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `USER_CLASSES` | `src/runtime/class/mod.rs:135` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CALLABLE_REGISTRY` | `src/runtime/class/mod.rs:139` | thread_local | 27 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `SLOTS_REGISTRY` | `src/runtime/class/mod.rs:146` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `OWN_SLOTS_REGISTRY` | `src/runtime/class/mod.rs:152` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `DICT_SUPPRESSED` | `src/runtime/class/mod.rs:155` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `KWARGS_REGISTRY` | `src/runtime/class/mod.rs:159` | thread_local | 6 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NAMEDTUPLE_BASE_SHAPES` | `src/runtime/class/mod.rs:179` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `METHOD_CACHE` | `src/runtime/class/mod.rs:183` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `METHOD_CACHE_GEN` | `src/runtime/class/mod.rs:187` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `SIMPLE_CLASS_CACHE` | `src/runtime/class/mod.rs:192` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `RUNTIME_CHECKABLE_PROTOCOLS` | `src/runtime/class/mod.rs:197` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ABC_VIRTUAL_SUBCLASSES` | `src/runtime/class/mod.rs:201` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `USER_ABC_OWN_ABSTRACT` | `src/runtime/class/mod.rs:207` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CODE_CLASS_REGISTERED` | `src/runtime/class/mod.rs:9480` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CLASS_DOCS` | `src/runtime/class/mod.rs:12396` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ABSTRACT_METHODS` | `src/runtime/class/mod.rs:14048` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CLOSURES` | `src/runtime/closure.rs:62` | thread_local | 8 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ACTIVE_CELLS` | `src/runtime/closure.rs:64` | thread_local | 11 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ACTIVE_MODULE_NAMES` | `src/runtime/closure.rs:66` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ACTIVE_QUALNAME_CONTEXTS` | `src/runtime/closure.rs:68` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_NAMES` | `src/runtime/closure.rs:797` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_QUALNAMES` | `src/runtime/closure.rs:799` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_DOCS` | `src/runtime/closure.rs:801` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_MODULES` | `src/runtime/closure.rs:803` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_ARGCOUNTS` | `src/runtime/closure.rs:810` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_VARNAMES` | `src/runtime/closure.rs:812` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_FLAGS` | `src/runtime/closure.rs:814` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_FREEVARS` | `src/runtime/closure.rs:816` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_PARAMS` | `src/runtime/closure.rs:821` | thread_local | 6 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_BOXED_PARAMS` | `src/runtime/closure.rs:823` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_RET_ANNOS` | `src/runtime/closure.rs:825` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_LINES` | `src/runtime/closure.rs:830` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_FILES` | `src/runtime/closure.rs:832` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CELLS` | `src/runtime/closure.rs:1477` | thread_local | 5 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GLOBAL_NAMESPACE` | `src/runtime/closure.rs:1733` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GLOBAL_ID_NAMESPACE` | `src/runtime/closure.rs:1735` | thread_local | 10 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `MISSING_GLOBAL_RAISES_NAME_ERROR` | `src/runtime/closure.rs:1737` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ACTIVE_MODULE_SYM_IDS` | `src/runtime/closure.rs:1751` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `MODULE_SYM_INFO` | `src/runtime/closure.rs:2155` | thread_local | 6 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `MODULE_FUNC_INFO` | `src/runtime/closure.rs:2159` | thread_local | 5 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FILES` | `src/runtime/file_io.rs:96` | thread_local | 14 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_FILE_ID` | `src/runtime/file_io.rs:98` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GC` | `src/runtime/gc.rs:56` | thread_local | 25 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GENERATORS` | `src/runtime/generator.rs:351` | thread_local | 19 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GEN_ACTIVE` | `src/runtime/generator.rs:355` | thread_local | 9 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `GEN_XFER` | `src/runtime/generator.rs:371` | thread_local | 10 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CALLER_CTX_STACK` | `src/runtime/generator.rs:381` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `RUNNING_GEN_STACK` | `src/runtime/generator.rs:387` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `LAST_STOP_VALUE` | `src/runtime/generator.rs:391` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `SHARED_CAPTURE` | `src/runtime/generator.rs:432` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HOOKS` | `src/runtime/integer_handle_registry.rs:46` | thread_local | 1 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `ITERATORS` | `src/runtime/iter.rs:168` | thread_local | 115 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `RANGE_ITERATOR_IDS` | `src/runtime/iter.rs:170` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_ITER_ID` | `src/runtime/iter.rs:175` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `STOP_ITERATION` | `src/runtime/iter.rs:178` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `MODULES` | `src/runtime/module.rs:29` | thread_local | 33 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `MODULE_VALUE_PTRS` | `src/runtime/module.rs:35` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SEARCH_PATHS` | `src/runtime/module.rs:37` | thread_local | 5 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NATIVE_FUNC_ADDRS` | `src/runtime/module.rs:42` | thread_local | 326 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NATIVE_TYPE_NAMES` | `src/runtime/module.rs:49` | thread_local | 8 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `VARIADIC_SYMBOL_IDS` | `src/runtime/module.rs:54` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `VARIADIC_FUNC_ADDRS` | `src/runtime/module.rs:59` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `KWARGS_SYMBOL_IDS` | `src/runtime/module.rs:62` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `KWARGS_FUNC_ADDRS` | `src/runtime/module.rs:66` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `BOXED_RETURN_SYMBOL_IDS` | `src/runtime/module.rs:76` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `BOXED_RETURN_FUNC_ADDRS` | `src/runtime/module.rs:80` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `SCRIPT_DIR` | `src/runtime/module.rs:92` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `CURRENT_MODULE_PACKAGE` | `src/runtime/module.rs:98` | thread_local | 11 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `NATIVE_TYPE_NAME_COLLISIONS` | `src/runtime/module.rs:106` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_ATTRS` | `src/runtime/pep695.rs:239` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ARRAYS` | `src/runtime/stdlib/array_mod.rs:515` | thread_local | 6 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ARRAY_IDS` | `src/runtime/stdlib/array_mod.rs:516` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ARRAY_TYPECODES` | `src/runtime/stdlib/array_mod.rs:517` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ARRAY_ORDER` | `src/runtime/stdlib/array_mod.rs:521` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ARRAY_REFCOUNTS` | `src/runtime/stdlib/array_mod.rs:525` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_ARRAY_ID` | `src/runtime/stdlib/array_mod.rs:529` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ATEXIT_HANDLERS` | `src/runtime/stdlib/atexit_mod.rs:55` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/cgi_mod.rs:253` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `INTERP_FUNCS` | `src/runtime/stdlib/code_mod.rs:122` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_INTERP_ID` | `src/runtime/stdlib/code_mod.rs:124` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SEARCH_PATH` | `src/runtime/stdlib/codecs_mod.rs:19` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CODEC_CACHE` | `src/runtime/stdlib/codecs_mod.rs:20` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ERROR_HANDLERS` | `src/runtime/stdlib/codecs_mod.rs:21` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CURRENT` | `src/runtime/stdlib/contextvars_mod.rs:58` | thread_local | 5 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `MISSING` | `src/runtime/stdlib/contextvars_mod.rs:62` | thread_local | 5 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `EMPTY_DATA` | `src/runtime/stdlib/contextvars_mod.rs:73` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `REDUCE_REGISTRY` | `src/runtime/stdlib/copyreg_mod.rs:31` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DISPATCH_TABLE` | `src/runtime/stdlib/copyreg_mod.rs:33` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `EXTENSION_REGISTRY` | `src/runtime/stdlib/copyreg_mod.rs:34` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `INVERTED_REGISTRY` | `src/runtime/stdlib/copyreg_mod.rs:35` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `EXTENSION_CACHE` | `src/runtime/stdlib/copyreg_mod.rs:36` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `PROFILERS` | `src/runtime/stdlib/cprofile_mod.rs:74` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_PROFILER_ID` | `src/runtime/stdlib/cprofile_mod.rs:75` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ACTIVE_STACK` | `src/runtime/stdlib/cprofile_mod.rs:78` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DIALECTS` | `src/runtime/stdlib/csv_mod.rs:25` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FIELD_SIZE_LIMIT` | `src/runtime/stdlib/csv_mod.rs:26` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NATIVE_PTRS` | `src/runtime/stdlib/ctypes_mod.rs:137` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/ctypes_mod.rs:219` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `PENDING_FIELDS` | `src/runtime/stdlib/dataclasses_mod.rs:108` | thread_local | 5 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `PENDING_OPTIONS` | `src/runtime/stdlib/dataclasses_mod.rs:112` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `DC_REGISTRY` | `src/runtime/stdlib/dataclasses_mod.rs:115` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `FIELD_TUPLES` | `src/runtime/stdlib/dataclasses_mod.rs:119` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `TZ_CLASS_ATTRS` | `src/runtime/stdlib/datetime_mod.rs:74` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DBM_STORES` | `src/runtime/stdlib/dbm_mod.rs:128` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DBM_NEXT_ID` | `src/runtime/stdlib/dbm_mod.rs:130` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECIMALS` | `src/runtime/stdlib/decimal_mod.rs:161` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECIMAL_IDS` | `src/runtime/stdlib/decimal_mod.rs:162` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `EXACT_VALUES` | `src/runtime/stdlib/decimal_mod.rs:167` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_DECIMAL_ID` | `src/runtime/stdlib/decimal_mod.rs:168` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECIMAL_REFCOUNTS` | `src/runtime/stdlib/decimal_mod.rs:170` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CTX_STACK` | `src/runtime/stdlib/decimal_mod.rs:2934` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DEFAULT_CTX` | `src/runtime/stdlib/decimal_mod.rs:2937` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/dev_tools_mod.rs:77` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ENUM_CLASSES` | `src/runtime/stdlib/enum_class.rs:97` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ENUM_KIND_MEMO` | `src/runtime/stdlib/enum_class.rs:100` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HAVE_ENUM_CLASSES` | `src/runtime/stdlib/enum_class.rs:106` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FRACTIONS` | `src/runtime/stdlib/fractions_mod.rs:118` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FRACTION_IDS` | `src/runtime/stdlib/fractions_mod.rs:119` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_FRACTION_ID` | `src/runtime/stdlib/fractions_mod.rs:120` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FRACTION_REFCOUNTS` | `src/runtime/stdlib/fractions_mod.rs:122` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TOTAL_ORDERING_SEEDS` | `src/runtime/stdlib/functools_mod.rs:440` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FUNC_WRAPPED` | `src/runtime/stdlib/functools_mod.rs:1903` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `COUNT_BASELINE` | `src/runtime/stdlib/gc_mod.rs:28` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `COLLECTIONS` | `src/runtime/stdlib/gc_mod.rs:30` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `GEN_TICKS` | `src/runtime/stdlib/gc_mod.rs:34` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FREEZE_COUNT` | `src/runtime/stdlib/gc_mod.rs:36` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DEBUG_FLAGS` | `src/runtime/stdlib/gc_mod.rs:38` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SORTERS` | `src/runtime/stdlib/graphlib_mod.rs:121` | thread_local | 9 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SORTER_ID` | `src/runtime/stdlib/graphlib_mod.rs:123` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SORTER_REFCOUNTS` | `src/runtime/stdlib/graphlib_mod.rs:128` | thread_local | 5 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HASHES` | `src/runtime/stdlib/hashlib_mod.rs:303` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HASH_IDS` | `src/runtime/stdlib/hashlib_mod.rs:304` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_HASH_ID` | `src/runtime/stdlib/hashlib_mod.rs:305` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HASH_REFCOUNTS` | `src/runtime/stdlib/hashlib_mod.rs:309` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HMACS` | `src/runtime/stdlib/hmac_mod.rs:212` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HMAC_IDS` | `src/runtime/stdlib/hmac_mod.rs:213` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_HMAC_ID` | `src/runtime/stdlib/hmac_mod.rs:214` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HMAC_REFCOUNTS` | `src/runtime/stdlib/hmac_mod.rs:218` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/http_mod.rs:73` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `KIND_SINGLETONS` | `src/runtime/stdlib/inspect_mod.rs:209` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `EMPTY_SINGLETON` | `src/runtime/stdlib/inspect_mod.rs:211` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `IMPLICIT_DEFAULT_SINGLETON` | `src/runtime/stdlib/inspect_mod.rs:213` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `IPS` | `src/runtime/stdlib/ipaddress_mod.rs:72` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `IP_IDS` | `src/runtime/stdlib/ipaddress_mod.rs:73` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_IP_ID` | `src/runtime/stdlib/ipaddress_mod.rs:74` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `IP_REFCOUNTS` | `src/runtime/stdlib/ipaddress_mod.rs:76` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ENCODERS` | `src/runtime/stdlib/json_mod.rs:63` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ENCODER_IDS` | `src/runtime/stdlib/json_mod.rs:64` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECODERS` | `src/runtime/stdlib/json_mod.rs:65` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECODER_IDS` | `src/runtime/stdlib/json_mod.rs:66` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_JSON_HANDLE_ID` | `src/runtime/stdlib/json_mod.rs:67` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `JSON_REFCOUNTS` | `src/runtime/stdlib/json_mod.rs:71` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CACHE` | `src/runtime/stdlib/linecache_mod.rs:76` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `LEVEL_TO_NAME` | `src/runtime/stdlib/logging_mod.rs:136` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NAME_TO_LEVEL` | `src/runtime/stdlib/logging_mod.rs:138` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `LOGGER_CACHE` | `src/runtime/stdlib/logging_mod.rs:140` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `LOGGER_CLASS` | `src/runtime/stdlib/logging_mod.rs:142` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `MANAGER_DISABLE` | `src/runtime/stdlib/logging_mod.rs:144` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RECORD_FACTORY` | `src/runtime/stdlib/logging_mod.rs:145` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `STDERR_CACHE` | `src/runtime/stdlib/logging_mod.rs:147` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/long_tail2_mod.rs:120` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/long_tail3_mod.rs:136` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ZI_CACHE` | `src/runtime/stdlib/long_tail3_mod.rs:1310` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/long_tail4_mod.rs:122` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DECOMPRESSORS` | `src/runtime/stdlib/lzma_mod.rs:256` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `COMPRESSORS` | `src/runtime/stdlib/lzma_mod.rs:258` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_LZMA_ID` | `src/runtime/stdlib/lzma_mod.rs:260` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `USER_TYPES` | `src/runtime/stdlib/mimetypes_mod.rs:42` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `MMAPS` | `src/runtime/stdlib/mmap_mod.rs:59` | thread_local | 9 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_MMAP_ID` | `src/runtime/stdlib/mmap_mod.rs:60` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NUMBERS_ABC_RANKS` | `src/runtime/stdlib/numbers_mod.rs:25` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FD_TABLE` | `src/runtime/stdlib/os_mod.rs:2510` | thread_local | 7 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_FD` | `src/runtime/stdlib/os_mod.rs:2512` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `PICKLE_GLOBAL_REGISTRY` | `src/runtime/stdlib/pickle_mod.rs:23` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CACHE` | `src/runtime/stdlib/platform_mod.rs:270` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RANDOMS` | `src/runtime/stdlib/random_mod.rs:62` | thread_local | 8 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RANDOM_IDS` | `src/runtime/stdlib/random_mod.rs:63` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_RANDOM_ID` | `src/runtime/stdlib/random_mod.rs:64` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RANDOM_REFCOUNTS` | `src/runtime/stdlib/random_mod.rs:66` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `DEFAULT_HANDLE` | `src/runtime/stdlib/random_mod.rs:69` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `GAUSS_SPARE` | `src/runtime/stdlib/random_mod.rs:72` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SAVED_STATES` | `src/runtime/stdlib/random_mod.rs:1175` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_STATE_ID` | `src/runtime/stdlib/random_mod.rs:1177` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RE_CACHE` | `src/runtime/stdlib/re_mod.rs:29` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `HANDLERS` | `src/runtime/stdlib/signal_mod.rs:100` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `WAKEUP_FD` | `src/runtime/stdlib/signal_mod.rs:101` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CONNS` | `src/runtime/stdlib/sqlite3_mod.rs:883` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CURSORS` | `src/runtime/stdlib/sqlite3_mod.rs:886` | thread_local | 8 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `IN_TX` | `src/runtime/stdlib/sqlite3_mod.rs:888` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SQ_NEXT_ID` | `src/runtime/stdlib/sqlite3_mod.rs:890` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RAND_STATE` | `src/runtime/stdlib/ssl_mod.rs:137` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/ssl_mod.rs:240` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RECURSION_LIMIT` | `src/runtime/stdlib/sys_mod.rs:245` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RECURSION_STATE_PTRS` | `src/runtime/stdlib/sys_mod.rs:247` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SWITCH_INTERVAL` | `src/runtime/stdlib/sys_mod.rs:249` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `INTERN_TABLE` | `src/runtime/stdlib/sys_mod.rs:250` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ASYNCGEN_HOOKS` | `src/runtime/stdlib/sys_mod.rs:252` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CACHE` | `src/runtime/stdlib/sysconfig_mod.rs:204` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TEMPDIR` | `src/runtime/stdlib/tempfile_mod.rs:291` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `THREAD_NAME` | `src/runtime/stdlib/threading_mod.rs:405` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `PROFILE_FN` | `src/runtime/stdlib/threading_mod.rs:406` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TRACE_FN` | `src/runtime/stdlib/threading_mod.rs:407` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TRACE_PROFILE_HOOK_ACTIVE` | `src/runtime/stdlib/threading_mod.rs:408` | thread_local | 4 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `STACK_SIZE` | `src/runtime/stdlib/threading_mod.rs:409` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CURRENT_IDENT` | `src/runtime/stdlib/threading_mod.rs:416` | thread_local | 7 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `LIVE_THREADS` | `src/runtime/stdlib/threading_mod.rs:427` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `WORKER_STDLIB_READY` | `src/runtime/stdlib/threading_mod.rs:436` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `MAIN_THREAD` | `src/runtime/stdlib/threading_mod.rs:1449` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TEST_TRACE_EVENTS` | `src/runtime/stdlib/threading_mod.rs:2132` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TEST_TRACE_RETURN_ARGS` | `src/runtime/stdlib/threading_mod.rs:2134` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TEST_GLOBAL_TRACE_RETURN` | `src/runtime/stdlib/threading_mod.rs:2136` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TZ_SNAPSHOT` | `src/runtime/stdlib/time_mod.rs:85` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `COROUTINE_FUNCTIONS` | `src/runtime/stdlib/types_mod.rs:63` | thread_local | 2 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `COROUTINE_GENERATOR_ORIGINS` | `src/runtime/stdlib/types_mod.rs:65` | thread_local | 3 | reachable | context-owned | ExecutionContext field / sub-aggregate |
| `SPECIAL_FORMS` | `src/runtime/stdlib/typing_mod.rs:713` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `TYPING_ALIAS_REGISTERED` | `src/runtime/stdlib/typing_mod.rs:715` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `UUIDS` | `src/runtime/stdlib/uuid_mod.rs:201` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `UUID_IDS` | `src/runtime/stdlib/uuid_mod.rs:202` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_UUID_ID` | `src/runtime/stdlib/uuid_mod.rs:203` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `UUID_REFCOUNTS` | `src/runtime/stdlib/uuid_mod.rs:205` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `UUID_INTERN` | `src/runtime/stdlib/uuid_mod.rs:211` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `STABLE_NODE` | `src/runtime/stdlib/uuid_mod.rs:215` | thread_local | 0 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SAFE_UUID_UNKNOWN_MEMBER` | `src/runtime/stdlib/uuid_mod.rs:217` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FILTERS` | `src/runtime/stdlib/warnings_mod.rs:84` | thread_local | 3 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `REGISTRY` | `src/runtime/stdlib/warnings_mod.rs:89` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `RECORD_LIST` | `src/runtime/stdlib/warnings_mod.rs:94` | thread_local | 4 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SNAPSHOTS` | `src/runtime/stdlib/warnings_mod.rs:98` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `WEAKREF_REGISTRY` | `src/runtime/stdlib/weakref_mod.rs:80` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `FINALIZE_REGISTRY` | `src/runtime/stdlib/weakref_mod.rs:85` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/webbrowser_mod.rs:176` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/wsgiref_mod.rs:68` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NS_PREFIXES` | `src/runtime/stdlib/xml_mod.rs:1262` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `NEXT_SHELL_SLOT` | `src/runtime/stdlib/xmlrpc_mod.rs:68` | thread_local | 12 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ZF_STORES` | `src/runtime/stdlib/zipfile_mod.rs:592` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `ZF_NEXT_ID` | `src/runtime/stdlib/zipfile_mod.rs:594` | thread_local | 1 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `SURROGATE_STRINGS` | `src/runtime/string_ops.rs:16` | thread_local | 2 | NONE | context-owned | ExecutionContext field / sub-aggregate |
| `CACHED_ISA` | `src/codegen/cranelift/jit.rs:42` | static_LazyLock | 0 | NONE | process-immutable | process-global cache (outside aggregate) |
| `CACHED_RT_SYMBOLS` | `src/codegen/cranelift/jit.rs:62` | static_LazyLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `COROUTINES` | `src/runtime/async_rt.rs:147` | static_LazyLock | 28 | reachable | process-immutable | process-global cache (outside aggregate) |
| `COMPLETED_COROUTINES` | `src/runtime/async_rt.rs:155` | static_LazyLock | 3 | reachable | process-immutable | process-global cache (outside aggregate) |
| `TASKS` | `src/runtime/async_rt.rs:159` | static_LazyLock | 16 | reachable | process-immutable | process-global cache (outside aggregate) |
| `WAKERS` | `src/runtime/async_task.rs:699` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `TIMERS` | `src/runtime/async_task.rs:1289` | static_LazyLock | 3 | NONE | process-immutable | process-global cache (outside aggregate) |
| `EXEC_FUNCTIONS` | `src/runtime/builtins/eval_exec.rs:573` | static_LazyLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `TYPE_OBJECT_STATE` | `src/runtime/builtins/type_objects.rs:133` | static_LazyLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `EMPTY_LINETABLE` | `src/runtime/class/mod.rs:9488` | thread_local | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `DICT_VERSIONS` | `src/runtime/dict_ops.rs:21` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `REAL_OPS` | `src/runtime/registry_bridge.rs:190` | static_plain | 0 | NONE | process-immutable | process-global cache (outside aggregate) |
| `TCP_SERVERS` | `src/runtime/stdlib/asyncio_mod.rs:18` | static_LazyLock | 3 | NONE | process-immutable | process-global cache (outside aggregate) |
| `EMPTY` | `src/runtime/stdlib/builtins_mod.rs:1291` | static_plain | 0 | NONE | process-immutable | process-global cache (outside aggregate) |
| `TZ_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1042` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `WEEKDAY_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1105` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `LOOSE_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1106` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `ISO_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1147` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `TOKEN_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1214` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `QUOTED_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1215` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `VALUE_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1216` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `ESCAPE_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1217` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `JUNK_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1218` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `WORD_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1269` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `JOIN_ESCAPE_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1270` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `SPLIT_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1313` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `ESCAPED_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1380` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `IPV4_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1391` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `CUT_PORT_RE` | `src/runtime/stdlib/http_cookiejar_mod.rs:1701` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `PIPES` | `src/runtime/stdlib/multiprocessing_mod.rs:32` | static_LazyLock | 5 | NONE | process-immutable | process-global cache (outside aggregate) |
| `PENDING_PROCESSES` | `src/runtime/stdlib/multiprocessing_mod.rs:34` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `QUEUES` | `src/runtime/stdlib/queue_mod.rs:67` | static_LazyLock | 9 | NONE | process-immutable | process-global cache (outside aggregate) |
| `QUEUE_IDS` | `src/runtime/stdlib/queue_mod.rs:69` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `QUEUE_REFCOUNTS` | `src/runtime/stdlib/queue_mod.rs:78` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `HISTORY` | `src/runtime/stdlib/readline_mod.rs:22` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `LEN` | `src/runtime/stdlib/readline_mod.rs:28` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `L` | `src/runtime/stdlib/readline_mod.rs:414` | static_OnceLock | 4 | NONE | process-immutable | process-global cache (outside aggregate) |
| `IDENTITY_DECORATOR_ADDR` | `src/runtime/stdlib/reprlib_mod.rs:741` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `LOCK_STATES` | `src/runtime/stdlib/threading_mod.rs:914` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `BARRIERS` | `src/runtime/stdlib/threading_mod.rs:1137` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `MONO_EPOCH` | `src/runtime/stdlib/time_mod.rs:632` | thread_local | 0 | NONE | process-immutable | process-global cache (outside aggregate) |
| `SNAPSHOT` | `src/runtime/stdlib/tracemalloc_mod.rs:23` | static_LazyLock | 2 | NONE | process-immutable | process-global cache (outside aggregate) |
| `OBJECT_TRACEBACKS` | `src/runtime/stdlib/tracemalloc_mod.rs:28` | static_LazyLock | 4 | NONE | process-immutable | process-global cache (outside aggregate) |
| `CLEARED_OBJECT_TRACEBACKS` | `src/runtime/stdlib/tracemalloc_mod.rs:30` | static_LazyLock | 4 | NONE | process-immutable | process-global cache (outside aggregate) |
| `STRING_HASH_STATE` | `src/runtime/string_ops.rs:19` | static_OnceLock | 1 | NONE | process-immutable | process-global cache (outside aggregate) |
| `JIT_LOCK` | `src/codegen/cranelift/jit.rs:34` | static_LazyLock | 34 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_CORO_ID` | `src/runtime/async_rt.rs:182` | static_atomic | 2 | reachable | process-service | explicit service handle (outside aggregate) |
| `NEXT_TASK_ID` | `src/runtime/async_rt.rs:185` | static_atomic | 2 | reachable | process-service | explicit service handle (outside aggregate) |
| `GATHER_OBSERVED_INCOMPLETE_HOOK` | `src/runtime/async_task.rs:1151` | static_atomic | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_EXEC_FUNCTION_ID` | `src/runtime/builtins/eval_exec.rs:575` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_CLASS_RUNTIME_KEY` | `src/runtime/class/mod.rs:217` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_GEN_ID` | `src/runtime/generator.rs:400` | static_atomic | 2 | reachable | process-service | explicit service handle (outside aggregate) |
| `UAF_DETECTOR_ARMED` | `src/runtime/rc.rs:1291` | static_atomic | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `UAF_DETECTOR_ENV_CHECKED` | `src/runtime/rc.rs:1294` | static_plain | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `ATTRIBUTE_GETTERS` | `src/runtime/registry_bridge.rs:159` | static_RwLock | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `CACHE_TOKEN` | `src/runtime/stdlib/abc_mod.rs:43` | static_atomic | 0 | NONE | process-service | explicit service handle (outside aggregate) |
| `PARALLEL_ACTIVE` | `src/runtime/stdlib/asyncio_mod.rs:1896` | static_atomic | 3 | NONE | process-service | explicit service handle (outside aggregate) |
| `PARALLEL_PEAK` | `src/runtime/stdlib/asyncio_mod.rs:1897` | static_atomic | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `FIRST_WEEKDAY` | `src/runtime/stdlib/calendar_mod.rs:69` | static_atomic | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `ONCE` | `src/runtime/stdlib/contextlib_mod.rs:1134` | static_plain | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_VAR_ID` | `src/runtime/stdlib/contextvars_mod.rs:108` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `ENABLED` | `src/runtime/stdlib/faulthandler_mod.rs:32` | static_atomic | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `ALL_THREADS` | `src/runtime/stdlib/faulthandler_mod.rs:33` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `DUMP_FD` | `src/runtime/stdlib/faulthandler_mod.rs:34` | static_plain | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `GRENT_LOCK` | `src/runtime/stdlib/grp_mod.rs:27` | static_Mutex | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_PIPE_ID` | `src/runtime/stdlib/multiprocessing_mod.rs:31` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `HOSTNAME_LOCK` | `src/runtime/stdlib/platform_mod.rs:546` | static_Mutex | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `PWENT_LOCK` | `src/runtime/stdlib/pwd_mod.rs:27` | static_Mutex | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_QUEUE_ID` | `src/runtime/stdlib/queue_mod.rs:70` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `BLOCKED_SIGNALS` | `src/runtime/stdlib/signal_mod.rs:103` | static_LazyLock | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `PENDING_SIGNALS` | `src/runtime/stdlib/signal_mod.rs:105` | static_LazyLock | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `HOSTNAME_ENV_LOCK` | `src/runtime/stdlib/socket_mod.rs:980` | static_Mutex | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `ONCE` | `src/runtime/stdlib/ssl_mod.rs:1966` | static_plain | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `COUNTER` | `src/runtime/stdlib/tempfile_mod.rs:180` | static_atomic | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `TYPE_PARAMS_MAKE_BASE_SEQ` | `src/runtime/stdlib/test_mod.rs:12` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_THREAD_IDENT` | `src/runtime/stdlib/threading_mod.rs:421` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `NEXT_BARRIER_ID` | `src/runtime/stdlib/threading_mod.rs:1136` | static_atomic | 2 | NONE | process-service | explicit service handle (outside aggregate) |
| `TRACING` | `src/runtime/stdlib/tracemalloc_mod.rs:14` | static_atomic | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `TRACED_CURRENT` | `src/runtime/stdlib/tracemalloc_mod.rs:15` | static_atomic | 7 | NONE | process-service | explicit service handle (outside aggregate) |
| `TRACED_PEAK` | `src/runtime/stdlib/tracemalloc_mod.rs:16` | static_atomic | 5 | NONE | process-service | explicit service handle (outside aggregate) |
| `NFRAME` | `src/runtime/stdlib/tracemalloc_mod.rs:17` | static_atomic | 1 | NONE | process-service | explicit service handle (outside aggregate) |
| `GC_BASELINE` | `src/runtime/stdlib/tracemalloc_mod.rs:20` | static_atomic | 3 | NONE | process-service | explicit service handle (outside aggregate) |

## Discarded rows

Test-only observability; no production ownership.

| symbol | path:line | storage | reason |
|---|---|---|---|
| `TEST_LOCK` | `src/codegen/cranelift/jit.rs:3546` | static_LazyLock | test-only hook counter |
| `ASYNC_STATE_TEST_LOCK` | `src/runtime/async_rt.rs:173` | static_Mutex | test-only hook counter |
| `HOOK_INVOKED` | `src/runtime/class/mod.rs:23259` | static_atomic | test-only hook counter |
| `HOOK_OWNER` | `src/runtime/class/mod.rs:23261` | static_atomic | test-only hook counter |
| `S1_HOOK_CALLED` | `src/runtime/class/mod.rs:25223` | static_atomic | test-only hook counter |
| `S4_GETITEM_CALLED` | `src/runtime/class/mod.rs:25324` | static_atomic | test-only hook counter |
| `S6_SET_NAME_CALLED` | `src/runtime/class/mod.rs:25394` | static_atomic | test-only hook counter |
| `S6_SET_NAME_OWNER` | `src/runtime/class/mod.rs:25395` | static_atomic | test-only hook counter |
| `R10_NO_KW_CALLED` | `src/runtime/class/mod.rs:26014` | static_atomic | test-only hook counter |
| `R11_INHERITED_CALLED` | `src/runtime/class/mod.rs:26183` | static_atomic | test-only hook counter |
| `INIT_CALLS` | `src/runtime/class/mod.rs:26743` | static_atomic | test-only hook counter |
| `INIT_RECEIVER` | `src/runtime/class/mod.rs:26798` | static_atomic | test-only hook counter |
| `INIT_SUBCLASS_CALLS` | `src/runtime/class/mod.rs:26799` | static_atomic | test-only hook counter |
| `SET_CALLED` | `src/runtime/class/mod.rs:26905` | static_atomic | test-only hook counter |
| `SET_VALUE` | `src/runtime/class/mod.rs:26906` | static_atomic | test-only hook counter |
| `DELETE_CALLED` | `src/runtime/class/mod.rs:26907` | static_atomic | test-only hook counter |
| `SET_NAME_CALLED` | `src/runtime/class/mod.rs:26979` | static_atomic | test-only hook counter |
| `MISSING_CALLED` | `src/runtime/class/mod.rs:27055` | static_atomic | test-only hook counter |
| `EQ_CALLED` | `src/runtime/class/mod.rs:27114` | static_atomic | test-only hook counter |
| `GETITEM_CALLED` | `src/runtime/class/mod.rs:27158` | static_atomic | test-only hook counter |
| `CURRENT_TEST_NAME` | `src/runtime/rc.rs:1335` | static_RwLock | test-only hook counter |
| `TEST_HOOK_EVENTS` | `src/runtime/stdlib/bdb_mod.rs:522` | thread_local | test-only hook counter |
| `TRACE_TEST_LOCK` | `src/runtime/stdlib/tracemalloc_mod.rs:1184` | static_LazyLock | test-only hook counter |
| `FINALIZE_CALLBACK_COUNT` | `src/runtime/stdlib/weakref_mod.rs:1979` | static_atomic | test-only hook counter |
| `REF_CALLBACK_COUNT` | `src/runtime/stdlib/weakref_mod.rs:1980` | static_atomic | test-only hook counter |
| `REF_CALLBACK_ARG_BITS` | `src/runtime/stdlib/weakref_mod.rs:1981` | static_atomic | test-only hook counter |
