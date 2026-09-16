# Glossary — the campaign's ubiquitous language

One word per concept, used identically in tickets, oracles, verdicts, and
`prompt.tmpl`. When a term exists here, a ticket spends the term, not a
paragraph. `_Avoid_` lines list synonyms that must not appear — a second name
for one concept is a second concept to the reader.

`prompt.tmpl` inlines the five executor-facing terms (**witness**, **predicate**,
**fitted**, **selector**, **silent death**). The rest are supervisor-side and
stay here.

## Executor-facing

**witness**
: The item's own content that decides a question about it. For a ported fixture,
the executed tail — the text after `# --- test body ---`. The prologue above it
is imported CPython scaffolding that the fixture never runs, so it is not the
witness. *A predicate that never reads the witness has not decided anything.*
_Avoid_: test body (ambiguous — the whole `fn` includes the prologue), source,
content

**predicate**
: A decision expression that reads the **witness**. Anything else that selects
items is a list wearing a predicate's name.
_Avoid_: filter, classifier, rule (reserve **rule** for the candidate surface)

**fitted**
: A decision reached by naming its own answer — a typed file set, an id set, an
`if name == "…"`. Rejected on sight, however correct the count it produces. The
form that appears *after* a rejection, because the verdict named the misses and
transcribing them is the shortest path to a passing number.
_Avoid_: hardcoded, cheating, overfit

**selector**
: The one runnable file that reproduces the reported set when run standalone.
The script named in the acceptance criterion must be the script that ran.
_Avoid_: derivation script, generator

**silent death**
: An unpermitted command segment in headless mode. The run ends immediately —
no output, no error, no comment on the ticket. Names one failure so it needs
describing once.
_Avoid_: auto-deny, refusal, permission error

## Supervisor-side

**attributing predicate**
: A test attributes to a work root iff it *asserts* a behaviour the Promise
names. Exercising the behaviour is not asserting it. Operationally: **which
subsystem, if changed, flips this test?**

**asserted object** / **vehicle**
: What a test claims versus what it uses to make the claim. Valid syntax written
in order to assert a runtime value, a scope, or a code-object property is a
**vehicle**; the parser root takes only tests whose **asserted object** is
acceptance or rejection of the source.
_Avoid_: incidental use, exercised-but-not-asserted

**denominator**
: The count of tests attributing to one work root. The deliverable of Wave A.

**floor**
: A set the answer must contain, fixed in the oracle before dispatch. A
denominator missing its floor is wrong regardless of its size.

**band**
: The accept range in the oracle. Never published to the executor — a published
number becomes a target.

**oracle**
: The pre-dispatch file holding expected value, band, floor, mandatory drops,
and fabrication tells. Sha256 quoted in the verdict to prove it predated the run.

**drop** / **keep**
: The two sides of an attribution decision. *Both are claims.* A branch
attributing N of N has reported a **candidate surface** twice and a
**predicate** never — zero drops is a failed predicate until proven otherwise.

**candidate surface**
: The set a **rule** admits before any **predicate** runs. Derived over the whole
corpus, never from a typed list of paths.

**fabricated process**
: A report asserting work that no code performed — a count that is a loop
length, an "audit" whose decision is `if name == "<the name the verdict gave>"`,
an evidence variable extracted and never read. Distinct from a wrong answer:
the answer may be right and the report still untrue.
