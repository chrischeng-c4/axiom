# Task: Gather Reference Context for Group 'try-except-dict-set-parse' (Change 'mamba-p1-bugfix')

Issues: #1112_fix-mamba-try-except-block-with-dict-set-literal-t

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Existing Spec Structure

The following ASCII tree shows existing spec directories for the affected crate(s). Use this to plan spec_plan entries — prefer modifying existing files over creating new ones.

```
cclab-mamba
├── README.md
├── all-mamba-p0.md
├── codegen
│   ├── cranelift-aot.md
│   ├── cranelift-jit
│   ├── cranelift-jit.md
│   ├── cranelift.md
│   ├── jit-refcount.md
│   └── llvm.md
├── conductor-mamba-p0-remaining-spec.md
├── conductor-mamba-p0-spec.md
├── config
│   └── config-schema.md
├── driver
│   ├── compiler-driver.md
│   └── repl.md
├── ffi
│   ├── bindings-and-stubs.md
│   ├── c-parser-and-types.md
│   └── memory-and-safety.md
├── hir
│   └── hir.md
├── lexer
│   └── tokens-and-indent.md
├── lower
│   ├── ast-to-hir.md
│   ├── hir-to-mir
│   └── hir-to-mir.md
├── mamba-all-p1-spec.md
├── mamba-crate-wiring-and-schema-binding.md
├── mamba-p1-lang-features-spec.md
├── mir
│   └── mir.md
├── parser
│   ├── ast.md
│   ├── expressions.md
│   ├── patterns.md
│   └── statements.md
├── pattern-matching.md
├── resolve
│   └── name-resolution.md
├── runtime
│   ├── async.md
│   ├── bigint.md
│   ├── builtins.md
│   ├── bytes-ops.md
│   ├── class.md
│   ├── closure.md
│   ├── dict-ops.md
│   ├── exception.md
│   ├── file-io.md
│   ├── gc.md
│   ├── generator.md
│   ├── iter.md
│   ├── list-ops.md
│   ├── module.md
│   ├── set-ops.md
│   ├── string-ops
│   ├── string-ops.md
│   ├── symbols.md
│   ├── thread-safe-runtime.md
│   ├── tuple-ops.md
│   └── value-and-rc.md
├── source
│   └── source-and-diagnostics.md
├── stdlib
│   ├── archive-and-compression.md
│   ├── argparse.md
│   ├── collections.md
│   ├── concurrency.md
│   ├── database.md
│   ├── datetime.md
│   ├── diagnostics-utils.md
│   ├── enum-and-dataclasses.md
│   ├── fs-utils.md
│   ├── functools.md
│   ├── hashlib.md
│   ├── io.md
│   ├── itertools.md
│   ├── json.md
│   ├── logging.md
│   ├── markup.md
│   ├── math.md
│   ├── native-implementations.md
│   ├── network.md
│   ├── numeric.md
│   ├── operator-and-copy.md
│   ├── os.md
│   ├── pathlib.md
│   ├── random.md
│   ├── re.md
│   ├── struct-and-binary.md
│   ├── sys.md
│   ├── testing.md
│   ├── text-processing.md
│   ├── time.md
│   └── typing-and-inspect.md
├── testing
│   ├── conformance.md
│   ├── cpython-compliance.md
│   ├── mamba-binding-tests-spec.md
│   ├── mamba-py312-conformance.md
│   ├── stdlib-coverage-lower.md
│   ├── test-coverage-remaining.md
│   └── test-harness.md
└── types
    ├── generics-and-protocols.md
    ├── type-checker.md
    └── type-representations.md

```

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-p1-bugfix/groups/try-except-dict-set-parse/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## Output: spec_plan array

For each change spec that will be created:
- spec_id: identifier for the new change spec
- action: "modify" (copy existing) or "create" (new skeleton)
- main_spec_ref: target path in cclab/specs/ (REQUIRED — must include a named subfolder,
e.g. `crates/cclab-sdd/logic/foo.md`, not `crates/cclab-sdd/foo.md`)
- source: path of existing spec to copy (only for "modify")
- sections: array of section types this spec needs (see change-spec.md § Section Selection)

**Action preference**: Use `action: modify` for any file visible in the spec directory tree
above. Reserve `action: create` for genuinely new subsystems with no existing spec file.

## File Decomposition Rules

1. **One spec file = one logical unit** (service, module, component). Do NOT bundle unrelated concerns.
2. **No duplicate section types in one file** — if a feature needs two REST APIs (e.g., external + internal), split into two spec files, each with its own `rest-api` section.
3. **Spec path mirrors source path** — `src/api/external.rs` → `specs/interfaces/external-api.md`.
4. **Cross-file references** — related specs link via `refs` frontmatter and `$ref` in content.

## In-Scope Specs

### cclab-mamba
- `read_path:specs/crates/mamba/README.md`
- `read_path:specs/crates/mamba/all-mamba-p0.md`
- `read_path:specs/crates/mamba/codegen/cranelift-aot.md`
- `read_path:specs/crates/mamba/codegen/cranelift-jit.md`
- `read_path:specs/crates/mamba/codegen/cranelift.md`
- `read_path:specs/crates/mamba/codegen/jit-refcount.md`
- `read_path:specs/crates/mamba/codegen/llvm.md`
- `read_path:specs/crates/mamba/conductor-mamba-p0-remaining-spec.md`
- `read_path:specs/crates/mamba/conductor-mamba-p0-spec.md`
- `read_path:specs/crates/mamba/config/config-schema.md`
- `read_path:specs/crates/mamba/driver/compiler-driver.md`
- `read_path:specs/crates/mamba/driver/repl.md`
- `read_path:specs/crates/mamba/ffi/bindings-and-stubs.md`
- `read_path:specs/crates/mamba/ffi/c-parser-and-types.md`
- `read_path:specs/crates/mamba/ffi/memory-and-safety.md`
- `read_path:specs/crates/mamba/hir/hir.md`
- `read_path:specs/crates/mamba/lexer/tokens-and-indent.md`
- `read_path:specs/crates/mamba/lower/ast-to-hir.md`
- `read_path:specs/crates/mamba/lower/hir-to-mir.md`
- `read_path:specs/crates/mamba/mamba-all-p1-spec.md`
- `read_path:specs/crates/mamba/mamba-crate-wiring-and-schema-binding.md`
- `read_path:specs/crates/mamba/mamba-p1-lang-features-spec.md`
- `read_path:specs/crates/mamba/mir/mir.md`
- `read_path:specs/crates/mamba/parser/ast.md`
- `read_path:specs/crates/mamba/parser/expressions.md`
- `read_path:specs/crates/mamba/parser/patterns.md`
- `read_path:specs/crates/mamba/parser/statements.md`
- `read_path:specs/crates/mamba/pattern-matching.md`
- `read_path:specs/crates/mamba/resolve/name-resolution.md`
- `read_path:specs/crates/mamba/runtime/async.md`
- `read_path:specs/crates/mamba/runtime/bigint.md`
- `read_path:specs/crates/mamba/runtime/builtins.md`
- `read_path:specs/crates/mamba/runtime/bytes-ops.md`
- `read_path:specs/crates/mamba/runtime/class.md`
- `read_path:specs/crates/mamba/runtime/closure.md`
- `read_path:specs/crates/mamba/runtime/dict-ops.md`
- `read_path:specs/crates/mamba/runtime/exception.md`
- `read_path:specs/crates/mamba/runtime/file-io.md`
- `read_path:specs/crates/mamba/runtime/gc.md`
- `read_path:specs/crates/mamba/runtime/generator.md`
- `read_path:specs/crates/mamba/runtime/iter.md`
- `read_path:specs/crates/mamba/runtime/list-ops.md`
- `read_path:specs/crates/mamba/runtime/module.md`
- `read_path:specs/crates/mamba/runtime/set-ops.md`
- `read_path:specs/crates/mamba/runtime/string-ops.md`
- `read_path:specs/crates/mamba/runtime/symbols.md`
- `read_path:specs/crates/mamba/runtime/thread-safe-runtime.md`
- `read_path:specs/crates/mamba/runtime/tuple-ops.md`
- `read_path:specs/crates/mamba/runtime/value-and-rc.md`
- `read_path:specs/crates/mamba/source/source-and-diagnostics.md`
- `read_path:specs/crates/mamba/stdlib/archive-and-compression.md`
- `read_path:specs/crates/mamba/stdlib/argparse.md`
- `read_path:specs/crates/mamba/stdlib/collections.md`
- `read_path:specs/crates/mamba/stdlib/concurrency.md`
- `read_path:specs/crates/mamba/stdlib/database.md`
- `read_path:specs/crates/mamba/stdlib/datetime.md`
- `read_path:specs/crates/mamba/stdlib/diagnostics-utils.md`
- `read_path:specs/crates/mamba/stdlib/enum-and-dataclasses.md`
- `read_path:specs/crates/mamba/stdlib/fs-utils.md`
- `read_path:specs/crates/mamba/stdlib/functools.md`
- `read_path:specs/crates/mamba/stdlib/hashlib.md`
- `read_path:specs/crates/mamba/stdlib/io.md`
- `read_path:specs/crates/mamba/stdlib/itertools.md`
- `read_path:specs/crates/mamba/stdlib/json.md`
- `read_path:specs/crates/mamba/stdlib/logging.md`
- `read_path:specs/crates/mamba/stdlib/markup.md`
- `read_path:specs/crates/mamba/stdlib/math.md`
- `read_path:specs/crates/mamba/stdlib/native-implementations.md`
- `read_path:specs/crates/mamba/stdlib/network.md`
- `read_path:specs/crates/mamba/stdlib/numeric.md`
- `read_path:specs/crates/mamba/stdlib/operator-and-copy.md`
- `read_path:specs/crates/mamba/stdlib/os.md`
- `read_path:specs/crates/mamba/stdlib/pathlib.md`
- `read_path:specs/crates/mamba/stdlib/random.md`
- `read_path:specs/crates/mamba/stdlib/re.md`
- `read_path:specs/crates/mamba/stdlib/struct-and-binary.md`
- `read_path:specs/crates/mamba/stdlib/sys.md`
- `read_path:specs/crates/mamba/stdlib/testing.md`
- `read_path:specs/crates/mamba/stdlib/text-processing.md`
- `read_path:specs/crates/mamba/stdlib/time.md`
- `read_path:specs/crates/mamba/stdlib/typing-and-inspect.md`
- `read_path:specs/crates/mamba/testing/conformance.md`
- `read_path:specs/crates/mamba/testing/cpython-compliance.md`
- `read_path:specs/crates/mamba/testing/mamba-binding-tests-spec.md`
- `read_path:specs/crates/mamba/testing/mamba-py312-conformance.md`
- `read_path:specs/crates/mamba/testing/stdlib-coverage-lower.md`
- `read_path:specs/crates/mamba/testing/test-coverage-remaining.md`
- `read_path:specs/crates/mamba/testing/test-harness.md`
- `read_path:specs/crates/mamba/types/generics-and-protocols.md`
- `read_path:specs/crates/mamba/types/type-checker.md`
- `read_path:specs/crates/mamba/types/type-representations.md`


Read these specs using the Read tool (file paths under `/Users/chrischeng/projects/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/mamba-p1-bugfix/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context mamba-p1-bugfix cclab/changes/mamba-p1-bugfix/payloads/create-reference-context.json
```