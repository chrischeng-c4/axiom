---
change: mamba-p2
date: 2026-02-22
---

# Context Clarifications


## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for re module?
- **Answer**: Rust-backed runtime stubs wrapping the regex crate. Implement as re_mod.rs in stdlib. Functions: re.compile, re.match, re.search, re.findall, re.sub, re.split, re.escape. Same pattern as existing stdlib modules.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for datetime/time?
- **Answer**: Rust-backed stubs using chrono crate. Implement as datetime_mod.rs. Classes: datetime, date, time, timedelta. Key methods: now(), strftime(), strptime(), timedelta arithmetic. Register in module system.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for collections?
- **Answer**: Rust-backed stubs. Implement as collections_mod.rs. Types: defaultdict (dict with factory), Counter (dict subclass with counting), deque (VecDeque wrapper), OrderedDict (insertion-ordered dict). Register in module system.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for itertools?
- **Answer**: Rust-backed iterator combinators. Implement as itertools_mod.rs. Functions: chain, islice, zip_longest, product, permutations, combinations, count, cycle, repeat, starmap, accumulate, groupby. Return lazy iterators via MbValue.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for functools?
- **Answer**: Rust-backed stubs. Implement as functools_mod.rs. Functions: partial (closure wrapping), lru_cache (HashMap-based memoization), reduce (fold), wraps (decorator helper), cached_property. Register in module system.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for pathlib?
- **Answer**: Rust-backed stubs using std::path. Implement as pathlib_mod.rs. Classes: Path, PurePath. Methods: exists, is_file, is_dir, name, stem, suffix, parent, joinpath, read_text, write_text, mkdir, iterdir, glob, resolve. Register in module system.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for random?
- **Answer**: Rust-backed stubs using rand crate. Implement as random_mod.rs. Functions: random, randint, randrange, choice, shuffle, sample, uniform, seed. Use thread_rng for default generator.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for dataclasses?
- **Answer**: Implement as dataclasses_mod.rs. The @dataclass decorator auto-generates __init__, __repr__, __eq__ from class fields. Support field(), frozen, order params. Works with existing class system — modify class registration to inject generated methods.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for dict/list unpacking?
- **Answer**: Parser + codegen change. Parse {**d1, **d2} and [*a, *b] as StarUnpack AST nodes. Lower to MIR calls: mb_dict_merge for dict unpacking, mb_list_extend for list unpacking. Add runtime helpers in dict_ops.rs and list_ops.rs.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for frozenset?
- **Answer**: Add ObjData::FrozenSet(HashSet) variant to rc.rs. Implement frozenset_ops.rs with immutable set operations (union, intersection, difference, symmetric_difference, issubset, issuperset, contains). No mutation methods. Register in symbols.rs.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for __slots__?
- **Answer**: Modify class registration in class.rs to support __slots__ declaration. When __slots__ is defined, restrict instance __dict__ to only listed attributes. Store slots list in MbClass. Reject attribute assignment for non-slot names. Register helpers in symbols.rs.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for enum module?
- **Answer**: Implement as enum_mod.rs. Support Enum base class, auto() value generation, member access by name/value, iteration over members. Store enum members in class registry with special __members__ dict. IntEnum, StrEnum as variants.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for contextlib?
- **Answer**: Implement as contextlib_mod.rs. Functions: contextmanager (decorator that wraps generator as context manager), suppress (suppress specific exceptions), redirect_stdout/stderr, closing, nullcontext. Depends on existing context manager protocol (#385).
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for copy module?
- **Answer**: Implement as copy_mod.rs. copy() does shallow copy (clone top-level), deepcopy() does recursive clone. Handle List, Dict, Set, Instance types. Support __copy__ and __deepcopy__ dunder protocols on instances.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for io/struct modules?
- **Answer**: io_mod.rs: StringIO and BytesIO in-memory stream classes wrapping Vec<u8>/String. Support read, write, seek, getvalue. struct_mod.rs: pack/unpack/calcsize for binary data using format strings. Depends on bytes type (#405).
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for hashlib?
- **Answer**: Implement as hashlib_mod.rs. Use Rust sha2/md5 crates. Functions: md5, sha1, sha256, sha512 constructors returning hash objects. Hash objects support update(data), digest(), hexdigest(). Depends on bytes type (#405).
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for __format__ protocol?
- **Answer**: Add __format__ dunder dispatch in class.rs method resolution. For f-string debug syntax f'{x=}', modify parser to emit DebugFString AST node that includes variable name prefix. Extend existing fstring codegen in hir_to_mir.rs.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for __del__ finalizer?
- **Answer**: Add __del__ support to GC system. When an object with __del__ is collected, invoke the destructor before freeing. Store destructor function pointer in MbObjectHeader. Handle prevent-resurrection semantics (if __del__ re-references object, skip collection).
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for exception groups (PEP 654)?
- **Answer**: Add ExceptionGroup class to exception.rs. Parse except* syntax in parser (new AST node). Lower to MIR with multi-handler matching. ExceptionGroup wraps Vec<MbValue> of sub-exceptions. Support split() and subgroup() methods.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for shutil?
- **Answer**: Implement as shutil_mod.rs using std::fs. Functions: copy, copy2, copytree, rmtree, move (rename), which (find executable in PATH), disk_usage. High-level wrappers around os/fs operations.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for tempfile?
- **Answer**: Implement as tempfile_mod.rs using Rust tempfile crate. Functions: NamedTemporaryFile, TemporaryDirectory, mkstemp, mkdtemp, gettempdir. Return file handles / path strings. Auto-cleanup via context manager or __del__.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for glob module?
- **Answer**: Implement as glob_mod.rs using Rust glob crate. Functions: glob(pattern), iglob(pattern). Return list of matching file paths as strings. Support *, ?, ** recursive patterns.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for traceback module?
- **Answer**: Implement as traceback_mod.rs. Functions: format_exc, format_exception, print_exc, print_exception, extract_tb, format_tb. Works with exception.rs stack trace data. Store frame info (file, line, function) in exception objects.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for warnings module?
- **Answer**: Implement as warnings_mod.rs. Functions: warn, warn_explicit, filterwarnings, simplefilter, resetwarnings. Warning categories: UserWarning, DeprecationWarning, FutureWarning, etc. Thread-local filter stack. Default prints to stderr.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for decimal/fractions?
- **Answer**: decimal_mod.rs: Decimal class wrapping Rust rust_decimal crate. Support arithmetic, comparison, quantize, to_eng_string. fractions_mod.rs: Fraction class with numerator/denominator as i64. GCD-based simplification. Mixed arithmetic with int/float.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for operator module?
- **Answer**: Implement as operator_mod.rs. Functions: add, sub, mul, truediv, floordiv, mod, pow, neg, pos, abs, eq, ne, lt, le, gt, ge, not_, and_, or_, xor, itemgetter, attrgetter, methodcaller. Thin wrappers around existing runtime ops.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for weakref module?
- **Answer**: Implement as weakref_mod.rs. Add weak reference support to GC system. weakref.ref(obj) creates a weak pointer that doesn't prevent collection. WeakValueDictionary and WeakSet. Callback on finalization. Requires GC integration for invalidation.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for inspect module?
- **Answer**: Implement as inspect_mod.rs. Functions: isfunction, isclass, ismethod, getmembers, signature, getsource (stub), getfile. Inspect class registry and closure metadata. Return MbValue wrappers for Parameter/Signature objects.
- **Rationale**:

## Additional Clarifications

### Q1: General
- **Question**: Implementation approach for base64 module?
- **Answer**: Implement as base64_mod.rs. Use Rust base64 crate. Functions: b64encode, b64decode, urlsafe_b64encode, urlsafe_b64decode, b32encode, b32decode, b16encode, b16decode. Input/output as bytes objects. Depends on bytes type (#405).
- **Rationale**: 

