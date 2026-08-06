# Metrics Prometheus Capabilities

## Brief

`metrics-prometheus` owns the two things every axiom service needs to expose a
`/metrics` endpoint and nothing else: integer accumulators that are safe to
touch from any thread on a request path, and the encoder that turns them into
Prometheus text exposition format 0.0.4.

It does not own metric names, help text, bucket boundaries, or scrape wiring — a
caller decides what to measure and what to call it. This crate decides that the
number a scraper reads is exactly the number the service accumulated, and that
the bytes around that number cannot be bent by anything a caller puts in a label.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what `metrics-prometheus` fundamentally does:
  accumulate observations without locks or floats, and encode them into the
  exposition format a scraper will accept.
- **Non-Core Features** keep the exposition channel honest — a label value is
  data and can never become structure. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Lock-Free Accumulation | - | implemented | verified | smoke | ready | core; every observation is an integer atomic operation, so a metric never costs a lock and never loses a count under concurrency |
| Exposition Encoding | - | implemented | verified | smoke | ready | core; the encoder emits HELP, TYPE and value lines in caller order, cumulative histogram buckets, and a sum scaled by exact integer arithmetic rather than floating point |
| Label Value Containment | - | implemented | verified | smoke | ready | non-core; a label value carrying a quote, a backslash, or a newline is escaped into its own field and can never close the label set or start a new sample line |

### Core Features

#### Lock-Free Accumulation

ID: lock-free-accumulation
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A counter only ever moves forward, a gauge holds exactly the last value set, and
a latency records a sum and a count that agree with the observations fed to it. A
histogram assigns each observation to the first bucket whose upper bound it does
not exceed, adds its value to the sum, and increments the total count — an
observation above every declared bound is still counted and still summed, it
simply lands in no stored bucket. Every one of those operations is an integer
atomic, so no observation path takes a lock and no observation is lost when two
threads record at once.
Surfaces:
- Rust API: `metrics_prometheus::Counter` - monotonic accumulation by one or by n, and the current total.
- Rust API: `metrics_prometheus::Gauge` - set and read a point-in-time value.
- Rust API: `metrics_prometheus::Latency` - record one duration into a sum and count pair.
- Rust API: `metrics_prometheus::Histogram` - build over ascending bounds, observe, and read the total count.
- Rust API: `metrics_prometheus::Bucket` - one upper bound, carrying both the exposition label and the integer bound.
Rust internal: the relaxed-ordering atomics behind each primitive, and the scan that picks the first bound an observation fits.
EC Dimensions:
- behavior: `cargo test -p metrics-prometheus --lib` - accumulation is additive and order-independent, a gauge overwrites, and a histogram's per-bucket counts, sum, and total count match the observations that were fed to it.
- security: `cargo test -p metrics-prometheus --lib` - an observation is counted exactly once even when it exceeds every declared bound, the total count never disagrees with the sum of what was observed, and no accumulation path can move a counter backwards.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Integer-only accumulation | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; sums are accumulated as integers in the base unit the caller observes in, so the recorded total is exactly reproducible and does not depend on the order observations happened to arrive in |
| Exclusive bucket assignment | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; an observation increments exactly one stored bucket — the first whose upper bound it does not exceed — and an observation above the last bound increments none of them while still reaching the sum and the total count |
| Monotonic counters | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; a counter exposes only increment-by-one and add, so no caller can reduce a counter, and a gauge is the only primitive whose value may fall |

#### Exposition Encoding

ID: exposition-encoding
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Rendering the same samples twice produces the same bytes. Each metric emits its
`# HELP` line, then its `# TYPE` line, then its value lines, and metric families
appear in the order the caller gave them so a scrape config stays stable across
releases. A histogram renders cumulative bucket rows ending in `+Inf`, whose
value is the total observation count by definition, followed by the sum and the
count. The sum is converted from the observed base unit to the published unit by
exact integer arithmetic: a power-of-ten divisor produces an integer part and a
zero-padded remainder, and any other divisor degrades to plain integer division
rather than publishing a fraction that is not the real one.
Surfaces:
- Rust API: `metrics_prometheus::render` - encode unlabeled samples in caller order.
- Rust API: `metrics_prometheus::render_labeled` - encode labeled metric families in caller order.
- Rust API: `metrics_prometheus::Histogram::render` - encode cumulative buckets, the scaled sum, and the count.
- Rust API: `metrics_prometheus::Sample` / `SampleGroup` / `LabeledSample` - the shapes a caller hands the encoder.
Rust internal: the exact decimal scaling of a sum by a divisor, and the cumulative running total across bucket rows.
EC Dimensions:
- behavior: `cargo test -p metrics-prometheus --lib` - the line order per family is HELP, TYPE, then values; families and rows keep caller order; bucket rows are cumulative and end at the total count; a power-of-ten divisor renders a zero-padded fraction.
- security: `cargo test -p metrics-prometheus --lib` - the rendered sum is never floating-point rounded and never larger or smaller than the accumulated total implies, cumulative bucket values never decrease, a divisor that is not a power of ten degrades rather than emitting a misleading fraction, and a zero divisor does not abort the scrape.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Stable line layout | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; every family emits exactly one HELP line and one TYPE line before its value rows, and rendering a sample list twice is byte-identical, so a scraper never sees a family appear, move, or lose its declaration between scrapes |
| Cumulative bucket rows | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; stored per-bucket counts are exclusive and made cumulative only at render time, the `+Inf` row equals the total observation count, and the rendered count series equals that same total |
| Exact sum scaling | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; the published sum is derived by integer division plus a zero-padded remainder, so it is exact for a power-of-ten divisor, degrades to integer division for any other divisor, and never introduces the drift a float accumulator would |

### Non-Core Features

#### Label Value Containment

ID: label-value-containment
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A label value is data. A value containing a double quote, a backslash, or a
newline is escaped into the exposition format's own escape sequences, so it
cannot close its own label, close the label set, add a label the caller did not
ask for, or begin a new sample line. Labels within a row are emitted in a
canonical order — by name, then by value — so two rows carrying the same labels
render identically regardless of the order the caller built them in.
Surfaces:
- Rust API: `metrics_prometheus::escape_label_value` - escape one value for use in a custom metric family.
- Rust API: `metrics_prometheus::Label` - one name and value pair within a row.
Rust internal: the escape writer shared by the labeled renderer and the public escaping helper.
EC Dimensions:
- behavior: `cargo test -p metrics-prometheus --lib` - labels sort by name then value, a row with no labels renders without braces, and a value with no special character passes through unchanged.
- security: `cargo test -p metrics-prometheus --lib` - a value carrying a quote, a backslash, a newline, or a fully-formed injected label set produces exactly one sample line whose label count is the one the caller declared.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Canonical label order | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; labels are sorted by name and then by value at render time, so the same label set built in two different orders produces the same bytes and a scraper never sees a series identity change |
| Escaped label values | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; backslash, double quote, and newline are replaced by their exposition escape sequences, and the escaping helper the public API exposes is the same one the renderer uses, so a custom family cannot escape differently |
| One row per sample | change | - | implemented | verified | smoke | `cargo test -p metrics-prometheus --lib`; a labeled sample renders exactly one line no matter what its label values contain, so no injected value can add a series, and the rendered line count equals the number of rows the caller supplied |
