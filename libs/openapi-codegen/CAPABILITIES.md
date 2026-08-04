# Openapi Codegen Capabilities

## Brief

An OpenAPI document is a promise about a wire format. Turning that promise into
code someone can call is where three separate questions get answered badly:
what a schema becomes in the target language, what an operation is *called*,
and which version of that language the result is allowed to assume.

`openapi-codegen` answers all three once, for three languages. A
language-neutral intermediate representation — the parsed document, the
identifier registry, the schema-key to type-name map, and the per-operation IR
— is built exactly once per spec, and three emitters render it as TypeScript,
Python, or Rust. Adding a language means adding an emitter, not re-deciding
what `getPetById` is named or which response a `2XX` key selects.

The crate is deliberately narrow in two directions. It parses a **subset** of
OpenAPI 3.0 / 3.1 / 3.2 — only the keywords an emitter consumes — and tolerates
everything else rather than rejecting documents it does not fully understand.
And it has no version gate: `openapi` is an opaque string, so a 3.2 document
carrying the `query` path-item keyword and an `additionalOperations` map parses
exactly like a 3.0 one. Support for a keyword and refusal to crash on a keyword
are different bars, and this crate is explicit about which one each feature
meets.

`generate` is the pure core: spec text in, in-memory files out, no filesystem
access. `run` is the one place that touches disk, so an embedding CLI cannot
write generated output without also recording the target contract that
produced it.

## Capabilities

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

Capabilities split into two roots. **Core Features** are the generation
pipeline itself: the document subset, the naming law, the operation IR, and the
per-language type mapping. **Non-Core Features** are the contract surfaces an
embedder opts into — the versioned target profile, its policy file, and the
sidecar manifest written beside materialized output.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Tolerant OpenAPI Document Subset | 3380 | implemented | verified | smoke | ready | core; only emitter-consumed keywords are modeled, unknown ones are tolerated, and 3.0 `nullable` and 3.1 `["T","null"]` reconcile to one nullability answer |
| Deterministic Identifier Naming | 3380 | implemented | verified | smoke | ready | core; one word-splitting law feeds every case conversion, and a per-scope registry makes collisions deterministic instead of silently overwriting |
| Language-Neutral Operation IR | 3380 | implemented | verified | smoke | ready | core; six method keywords plus `additionalOperations` in a fixed order, path-level parameters merged ahead of operation-level ones, and one response-selection ladder |
| Per-Language Type Mapping | 3380 | implemented | verified | smoke | ready | core; three emitters map the same schema to their own type expression, each degrading to that language's free-form type rather than failing generation |
| Versioned Target Profiles | 3380 | implemented | verified | smoke | ready | non-core; an explicit profile per language with a deterministic requirements record, and a cross-language request refused before any parsing |
| Contained Output Materialization | 3380 | implemented | verified | smoke | ready | non-core; one write path that refuses a generated path escaping the output directory and emits the contract manifest beside explicitly targeted files |

### Core Features

#### Tolerant OpenAPI Document Subset

ID: tolerant-openapi-document-subset
Root WI: 3380
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
The document model deserializes only the keywords an emitter consumes and
tolerates every other field. That is a product decision, not an omission: real
specs carry annotation, validation, and vendor keywords no client generator
acts on, and a strict model would reject working documents over fields it would
have ignored anyway. `exclusiveMinimum` is a boolean in 3.0 and a number in
3.1; both parse and both are ignored.

There is no version gate. `openapi` is a bare string, never compared, so 3.0,
3.1, 3.2, and a document that omits the field entirely all parse. What changes
across versions is what is *modeled*: OpenAPI 3.2 adds the `query` path-item
keyword and the `additionalOperations` map, and both are first-class fields on
the path item rather than tolerated unknowns.

Two version differences are reconciled rather than passed through. `type` may
be a single string or an array of strings; both normalize to a list. And
nullability has two spellings — 3.0's `nullable: true` and 3.1's `"null"` entry
in the type array — which converge on one answer, with the `"null"` sentinel
stripped from the declared type list so a downstream emitter never sees it
twice.

A `$ref` and an inline value share one shape. The reference form is tried
first, and because its `$ref` field is required, a plain object without one
falls through to the inline form; that ordering is what makes an untagged
either-or work at all.

Surfaces:
- Rust API: `openapi_codegen::ir::openapi::Spec` - the parsed document root.
- Rust API: `openapi_codegen::ir::openapi::PathItem` - the six method keywords plus 3.2's `additionalOperations` and path-level parameters.
- Rust API: `openapi_codegen::ir::openapi::Schema::type_names` - declared types with the 3.1 `"null"` sentinel removed.
- Rust API: `openapi_codegen::ir::openapi::Schema::is_nullable` - one nullability answer across 3.0 and 3.1 spellings.
- Rust API: `openapi_codegen::ir::openapi::RefOr` - reference-or-inline, reference tried first.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - both nullability spellings converge, a single-string and an array `type` normalize alike, and a 3.2 document with `query` and `additionalOperations` parses.
- security: `cargo test -p openapi-codegen` - an unknown or wrongly-typed keyword is tolerated rather than aborting generation, so an untrusted spec cannot deny service by carrying a field the model does not know.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Nullability reconciles across 3.0 and 3.1 | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::openapi::tests::nullable_30_and_31_converge` asserts both spellings and that the declared type list drops `"null"` in both |
| Reference form wins only when `$ref` is present | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::openapi::tests::ref_or_picks_ref_when_dollar_ref_present` paired with `ref_or_picks_item_for_inline_schema` proves the fallthrough, so a one-sided assertion cannot pass a model that always picks the reference |
| Unknown keywords do not fail parsing | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::openapi::tests::tolerates_unknown_keywords_and_exclusive_minimum_variants` parses both the 3.0 boolean and the 3.1 numeric form |
| 3.2 keywords are modeled, not tolerated | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::openapi::tests::path_item_parses_32_query_keyword_and_additional_operations` reads the values back rather than only asserting the parse succeeded |

#### Deterministic Identifier Naming

ID: deterministic-identifier-naming
Root WI: 3380
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
Every generated identifier comes from one word-splitting law: non-alphanumeric
characters separate words, and a lowercase-or-digit followed by an uppercase
character starts a new one. The three case conversions — Pascal, camel, snake —
are then views over the same word list, so a name cannot be Pascal-cased one
way in a type declaration and camel-cased incompatibly in a function name.

The law has consequences worth stating rather than discovering. A run of
capitals is one word, so an acronym flattens: `HTTPServer` snake-cases to a
single token. An empty or fully-stripped input still has to produce a legal
identifier, and each conversion has its own documented placeholder rather than
returning an empty string. A name starting with a digit is prefixed instead of
being rejected, because refusing to generate is a worse outcome than generating
a mangled-but-legal name.

Collisions are made deterministic instead of impossible. A registry hands out
the requested name the first time and a numbered suffix every time after, so
two operations that reduce to the same base name both get a name and the second
one's name is stable across runs. Emitters reserve component type names in the
registry *before* asking it for per-operation names, so a synthesized
`XxxData` can never quietly shadow a schema the document declared.

Where a name cannot be made a legal identifier at all, the emitter is told:
property keys are quoted and escaped, and member access falls back to bracket
form.

Surfaces:
- Rust API: `openapi_codegen::ir::names::to_pascal` / `to_camel` / `to_snake` - the three case views over one word split.
- Rust API: `openapi_codegen::ir::names::is_ident` - whether a name can be emitted bare.
- Rust API: `openapi_codegen::ir::names::prop_key` / `param_access` - quoted-key and bracket-access fallbacks.
- Rust API: `openapi_codegen::ir::names::NameRegistry` - first-come naming with deterministic numbered suffixes.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - the three conversions agree on one word split, empty input yields each conversion's documented placeholder, and the registry's second request is suffixed.
- security: `cargo test -p openapi-codegen` - a spec-controlled schema key that is not a legal identifier is quoted and escaped rather than emitted bare, so a hostile key cannot break out of the generated declaration.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| One word split feeds every conversion | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::names::tests` asserts camelCase humps and acronym flattening on the same inputs across conversions |
| Illegal identifiers still produce a legal name | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::names::tests` pins the empty-input placeholder per conversion and the leading-digit prefix |
| Collisions are suffixed, not overwritten | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `emit::ts::plan::tests::per_op_type_names_avoid_component_collision` proves the reserved component name survives and the synthesized one is suffixed |
| Unquotable keys are escaped at the emission seam | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::names::tests` asserts backslash-then-quote escaping order for a property key |

#### Language-Neutral Operation IR

ID: language-neutral-operation-ir
Root WI: 3380
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
One pass turns a parsed document into an ordered list of operations every
emitter consumes identically. The order is fixed — paths in document key order,
then the six method keywords in a fixed sequence, then `additionalOperations`
whose method name is not already one of those six — so generated output is
byte-stable across runs and a method cannot be emitted twice under two
spellings.

Parameter assembly encodes three decisions. Path-level parameters are merged
**ahead of** the operation's own, so a path-wide parameter keeps a stable
position in the generated signature. A parameter located in the path is forced
required regardless of what the document claims, because a path template cannot
be rendered without it. Cookie parameters and parameters given as references
are dropped rather than half-modeled — the emitters have no place to put them,
and inventing one would generate code that does not work.

Request bodies come from `application/json` only, and a body given as a
reference is dropped for the same reason. Response selection is a ladder: the
explicit success codes first, then the lowest remaining `2`-prefixed key, then
`default`. The ladder is what makes a spec that documents only `204` or only
`2XX` still produce a typed response.

OpenAPI 3.2's `QUERY` method is a first-class member of the IR, not a special
case bolted onto `GET`. It is query-shaped for hook generation, and it carries
its POST-twin path: the `x-post-twin` vendor extension when present, otherwise
the same path template. Computing the twin once here is what lets all three
emitters expose the same runtime fallback without three copies of the policy.

Surfaces:
- Rust API: `openapi_codegen::ir::operations::build` - spec to ordered operation IR.
- Rust API: `openapi_codegen::ir::operations::OperationIR` - method, path, grouped parameters, body, response, and POST-twin path.
- Rust API: `openapi_codegen::ir::operations::ParamIR` - one parameter with its resolved required flag.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - keyword order and `additionalOperations` de-duplication hold, path-level parameters precede operation-level ones, a path parameter is required even when declared otherwise, and the response ladder picks each rung.
- security: `cargo test -p openapi-codegen` - a referenced parameter or body is dropped rather than emitted as an unresolved name, so a spec cannot inject an unresolvable symbol into generated code.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Method order and de-duplication are fixed | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::operations::tests` asserts the emitted method sequence and that an `additionalOperations` entry naming a keyword method is not duplicated |
| Path parameters are required by construction | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::operations::tests` declares a path parameter as not required and asserts the IR reports it required |
| The response ladder picks each rung | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::operations::tests` covers an explicit success code, a bare `2`-prefixed key, and `default`, so a single-case assertion cannot pass a ladder that only checks `200` |
| POST-twin resolution happens once | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `ir::operations::tests` asserts the `x-post-twin` override and the same-path default, and that a non-`QUERY` operation has no twin |

#### Per-Language Type Mapping

ID: per-language-type-mapping
Root WI: 3380
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
Three emitters map the same schema to their own type expression. They share the
keyword precedence — composition keywords first, then enumerations, then the
declared type — and they share the rule that an unresolvable reference degrades
to that language's free-form type rather than failing generation. Where they
differ, they differ because the target language differs, and each difference is
a decision rather than a gap.

TypeScript keeps the most structure: an intersection for `allOf`, a union for
`oneOf`/`anyOf`, a literal union for an enumeration, and an inline object type
with optional members and an index signature. Python has no intersection type,
so `allOf` falls back to its first member, and an inline object becomes a typed
mapping rather than a synthesized nested model. Rust has neither, so an
untagged union becomes the free-form value type and a string enumeration
becomes a plain string with the constraint dropped — a generated client that
deserializes is worth more than one that encodes a constraint it cannot
express.

Nullability is applied once, at the outer layer, and each language refuses to
double-wrap: an already-nullable expression, or that language's free-form type,
is returned unchanged. An array whose element type is a union is parenthesized
where the target language needs it.

Python's mapping additionally varies by target profile: with an explicit
profile it emits native union and optional syntax, and without one it emits the
legacy typing-module forms, so a profile-less caller's generated files stay
byte-identical to what they were before profiles existed.

Surfaces:
- Rust API: `openapi_codegen::emit::ts::tsmap::type_expr` / `object_expr` - the TypeScript type expression.
- Rust API: `openapi_codegen::emit::py::pymap::type_expr` / `object_expr` / `optional` / `union_expr` - the Python type expression, profile-aware.
- Rust API: `openapi_codegen::emit::rust::rsmap::type_expr` / `object_expr` / `optional` - the Rust type expression.
- Rust API: `openapi_codegen::ir::typemap::TypeMap` - the shared schema-key to type-name map the three share.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - primitives, formats, arrays, references, enumerations, and both nullability spellings map to the documented expression in each language, and each language's documented degradation is asserted rather than assumed.
- security: `cargo test -p openapi-codegen` - a reference outside the modeled component subset degrades to a free-form type instead of emitting an unresolved identifier into generated source.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| The three languages agree on keyword precedence | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `emit::ts::tsmap::tests::enum_one_of_all_of` with the `pymap` and `rsmap` primitive suites pins the shared ordering and each language's own degradation |
| Nullability is applied once | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `emit::ts::tsmap::tests::array_and_nullable` asserts both spellings and the parenthesized array-of-union form |
| An unresolvable reference degrades per language | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; the `tsmap`/`pymap`/`rsmap` reference cases assert the resolved name, and the fallback expression is the documented free-form type in each |
| Profile-less Python output is unchanged | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::python_profiles_record_requirements_and_use_their_supported_typing_syntax` asserts the profile syntax, so the legacy form is a separate observed case rather than an assumption |

### Non-Core Features

#### Versioned Target Profiles

ID: versioned-target-profiles
Root WI: 3380
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
A target profile is the explicit statement of which language version generated
code may assume. Selecting one is opt-in: with no profile the generated files
stay byte-for-byte what they were before profiles existed and no manifest is
written, so adopting the feature is never a silent output change.

Each profile carries a deterministic requirements record — compiler, minimum
version, language standard, module system and resolution and strictness where
the language has them, transport, and an ordered runtime dependency list. The
record is derived from the profile, not configured alongside it, so two callers
selecting the same profile cannot disagree about what it requires.

Profiles are where version-specific syntax lives. Python 3.12 and later emit
PEP 695 type aliases where 3.11 emits plain assignment; Rust 2024 escapes a
schema field named `gen` and renames it back over the wire where Rust 2021
emits it bare. TypeScript has one profile today and records only the compiler
floor — the emitter has no version-specific improvement that is safe to make,
and saying so is more useful than inventing a difference.

A profile belongs to exactly one language, and a request whose profile does not
match the requested language is refused **before** any parsing or generation,
so a mismatch surfaces as an error naming the profile rather than as generated
output in the wrong language. A profile passed explicitly alongside a
conflicting one in the options is refused the same way.

Projects pin defaults in a policy table keyed by language. A policy entry whose
value names a different language's profile is rejected when the table is read,
so a misconfigured default fails at configuration time rather than at the first
generation.

Surfaces:
- Rust API: `openapi_codegen::TargetProfile` - the per-language profile enumeration and its stable id.
- Rust API: `openapi_codegen::TargetRequirements` - the derived requirements record.
- Rust API: `openapi_codegen::TargetPolicy` - the `[targets]` table and its per-language resolution.
- Rust API: `openapi_codegen::generate_for_target` - generation under an explicit profile, with the cross-language refusal.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - each profile's id and requirements are exact, and the version-specific syntax differences appear only in the profiles that promise them.
- security: `cargo test -p openapi-codegen` - a cross-language profile is refused before parsing, and a policy entry naming another language's profile is refused when the table is read.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Requirements are derived, not configured | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::python_profiles_record_requirements_and_use_their_supported_typing_syntax` asserts minimum version and dependency list per profile |
| Version-specific syntax is profile-scoped | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::rust_2024_profile_escapes_gen_field_without_changing_rust_2021` asserts both editions in one test, so the escape cannot leak into 2021 unnoticed |
| A cross-language profile is refused early | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::target_profile_must_match_the_requested_language` asserts the error names the offending profile id |
| A policy default is validated at read time | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `target::tests` parses a `[targets]` table whose entry names another language's profile and asserts the rejection |

#### Contained Output Materialization

ID: contained-output-materialization
Root WI: 3380
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
Generated files reach disk through exactly one method. That is a containment
decision: a generated relative path that is absolute, or that contains a parent
or root component, is refused rather than joined, so a spec-derived path cannot
write outside the output directory the caller named. It is also a consistency
decision — every embedding CLI records the same contract instead of writing its
own file loop and silently dropping the target metadata.

Explicitly targeted output additionally writes a sidecar manifest naming the
generator, the compiler, the profile id, the language, the minimum version and
language standard, the module settings where the language has them, the
transport, and the ordered runtime dependencies. The manifest is what makes the
target contract inspectable after the in-memory result has been written and the
generating process is gone. Output generated without a profile writes no
sidecar, because there is no contract to record.

The command-line entry distinguishes its failures: a spec that cannot be read
exits differently from a spec that cannot be generated or written, so a caller
scripting the generator can tell a missing input from a bad one.

Surfaces:
- Rust API: `openapi_codegen::GeneratedOutput::write_to_dir` - the one write path, with the containment refusal.
- Rust API: `openapi_codegen::GeneratedOutput::manifest` - the sidecar contract record, present only for explicit targets.
- Rust API: `openapi_codegen::GenerationManifest` - the serialized manifest shape.
- Rust API: `openapi_codegen::run` - the CLI entry and its distinguished exit codes.

EC Dimensions:
- behavior: `cargo test -p openapi-codegen` - materialized targeted output contains the manifest with the exact profile id and minimum version, and untargeted output contains no sidecar.
- security: `cargo test -p openapi-codegen` - an absolute or parent-relative generated path is refused before any write, so generation cannot escape the named output directory.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| One write path records the contract | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::materialized_output_writes_a_versioned_contract_manifest` reads the manifest back off disk and asserts its target and minimum version |
| A path that escapes the output directory is refused | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; the `safe_output_path` cases assert the refusal for the absolute and parent-component forms rather than only the ordinary form succeeding |
| Untargeted output stays sidecar-free | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::generates_all_files` pins the exact untargeted file list, so an added sidecar would fail the assertion |
| Read failure is distinguishable from generation failure | change | 3380 | implemented | verified | smoke | `cargo test -p openapi-codegen`; `lib::tests::invalid_spec_is_an_error` covers the generation error, and `run` maps the read failure to its own exit code |
