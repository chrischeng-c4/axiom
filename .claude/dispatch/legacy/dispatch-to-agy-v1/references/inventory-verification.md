# Verifying inventory and classification reports

Use these rules when AGY derives a set, denominator, classification, site
inventory, or regional footprint.

## Decide whether to dispatch

Dispatch only when the task is:

- **recomputable**: derive the answer independently without reading AGY's
  report;
- **floor-bearing**: name at least one result that is impossible if the source
  was not inspected;
- **non-iterative**: AGY cannot simply grind a local success oracle to green;
- **single-writer**: its deliverable paths do not overlap another live task.

Scale concurrency by controller verification capacity, not AGY's process cap.
Start serial on a new or unstable channel, then increase only after the prior
batch is independently accepted.

## Bound the candidate surface

Derive the candidate surface independently before reading AGY's shortlist.
The report's own denominator cannot reveal an omitted suite, directory, or
producer family.

For a thing, inspect at least these axes:

- **consumer**: where is it read, set, or invoked?
- **provider**: what declares it or must already be true for it to work?
- **identity**: what local name, binding, handle, or resource id refers to it?

Independent authorship is not independent derivation. Two selectors built from
the same axis can agree exactly while sharing the same blind spot. State the
axis of every oracle route and prove that a corroborating route could have
disagreed; a strict subset is not corroboration.

Distinguish pointwise from regional surfaces:

- A field, call site, config key, or symbol use is pointwise. Require complete
  path:line rows.
- A subsystem, feature leg, or whole test footprint is regional. Require
  structural regions and a closure rule. Every inventoried site must fall in
  exactly one region, and each region should expose residual lines the
  point-selector did not find. Zero residual usually means the report renamed
  matched lines as regions.

If another discovery axis keeps appearing, stop extending the term list.
Change the deliverable from sites to structural regions.

## Audit both decisions

Treat an admission and a rejection as equally load-bearing claims.
Independently inspect both:

- false positives: admitted items whose own body/assertions do not exercise the
  promised behavior;
- false negatives: whole-corpus items that exhibit the behavior structurally
  but are absent from the result.

If an admission predicate never reads the item's body/code/assertions, it did
not make an item-level judgment. A branch with zero exclusions is provisional
until independently proven; blanket group rationale plus `N of N` usually
means the selector was reported twice.

Attribute a test to the subsystem whose implementation change would flip its
outcome. A test may use one API only as the vehicle for asserting another
subsystem's contract. Include assertion-free "does not raise" tests explicitly;
an assertion-reference predicate is structurally blind to them.

## Verify artifacts, rows, and prose

Prefer a machine-readable sidecar whose script also renders the terminal
report. Recompute from that sidecar; a number typed only in prose is not
evidence. Verify that the script computed each field rather than carrying
forward a value transcribed from a prior round.

For every subtotal, require complete member evidence. Representative examples
illustrate a family but do not authenticate its count. Validate every cited
path, line, enclosing function, type, and semantic attribution against fresh
source.

When the controller's recount and AGY's differ by a wide but stable factor,
compare predicates before arithmetic. Print the controller predicate and
denominator so the next revision can address a concrete disagreement.

## Revise by consequence

After rejection, treat named missing identities as examples, never as the
specification. Check the replacement selector for hard-coded identity lists.

Before ordering a form change, run its counterfactual against the corpus.
Reject a form defect when it changes the admitted set or leaves a systematic
blind spot. If the independently audited set is unchanged, carry the form
finding to the future gate ticket instead of spending another report round.

A one-off impurity may be accepted only when it is named and its effect is
quantified. A recurring family with one systematic cause requires revision.

For transformations such as spans, edits, or patches, apply the artifact to a
copy and run an external language/tool checker. A controller-authored boundary
rule detects only failure modes already anticipated by the controller. If no
independent checker exists, record that absence and keep the last-mile
transformation with the controller.
