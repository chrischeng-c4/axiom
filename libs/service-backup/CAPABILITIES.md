# Service Backup Capabilities

## Brief

`service-backup` is the shape around a snapshot, not the snapshot itself.
Consistency belongs to the data plane: a service's state machine produces
bytes at a concrete applied index, and `raft-runtime` owns snapshot install
and log compaction. What every axiom service then needs — and would otherwise
reinvent once per service — is the same small set of answers. Where do those
bytes go? How is the object named so age retention can find it again later?
What does an operator write in a CR to say "hourly, to this bucket, keep a
day"? And what happens when the runner binary was built without the adapter
that destination needs?

This crate owns those answers: the destination and policy schema, the
`BackupSink` trait with its local and GCS implementations, a runner primitive
that writes one object and applies retention, and the narrower exact-object
fetch that bootstrap and restore need. S3 is behind a crate feature; local and
GCS are always linked. The authenticated admin-snapshot transport that backup
CLIs use to *obtain* the bytes is behind the `http-client` feature.

It does not own scheduling — the operator translates a cron expression into a
Kubernetes CronJob — and it does not own snapshot consistency.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** are what an operator declares and what the runner does
  with it: parsing a destination, validating a policy, and writing one
  retained object.
- **Non-Core Features** keep that contract honest across builds and
  restores — failing loud when an adapter is missing, fetching one exact
  object back, and keeping the CLI's own documentation from drifting away
  from what the parser accepts. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Destination URI Contract | 3376 | implemented | verified | smoke | ready | core; one parser accepts `file://`, `s3://` and `gs://`, splits bucket from prefix, and rejects everything else with an error naming every scheme it does accept |
| CRD-Safe Scheduled Policy | 3376 | implemented | verified | smoke | ready | core; a flat operator-facing policy that a Kubernetes structural schema can hold, converted to the runtime policy through exactly one validated path |
| Retained Object Write | 3376 | implemented | verified | smoke | ready | core; one run writes a timestamp-named object, applies age retention, and reports the key, byte count and prune count it actually produced |
| Fail-Loud Unlinked Adapter | 3376 | implemented | verified | smoke | ready | non-core; a destination whose adapter is not in this build fails with the exact rebuild action rather than silently writing somewhere else |
| Exact Object Restore Fetch | 3376 | implemented | verified | smoke | ready | non-core; bootstrap and restore read one named object by URI, a deliberately narrower surface than a sink prefix |
| Non-Drifting Scheme Documentation | 3376 | implemented | verified | smoke | ready | non-core; the CLI topic renders the scheme table at call time from the same constant the parser uses, so the docs cannot outlive the parser |
| Authenticated Admin Snapshot Transport | 3376 | implemented | verified | smoke | ready | non-core; the standard bearer-authenticated fetch of a service's admin snapshot, with the failing status and body preserved in the diagnostic |

### Core Features

#### Destination URI Contract

ID: destination-uri-contract
Root WI: 3376
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
An operator writes one string. `BackupDestination::from_uri` is the only thing
that reads it, and it accepts exactly three schemes: `file://` for a local
path, `s3://` for an S3-compatible object store, and `gs://` for Google Cloud
Storage. For the two object-store forms the first path segment is the bucket
and the remainder is the prefix, with surrounding slashes trimmed; a URI whose
bucket is empty is rejected rather than silently producing a bucketless
request. Anything else is refused with an error that names every scheme this
build accepts, so an operator who typed `ftp://` learns the alternatives from
the failure instead of from the source. The accepted set is a single constant,
`SUPPORTED_SCHEMES`, which also records whether a live sink for that scheme is
linked into this build — and it records that from `cfg!`, not from a
hand-maintained boolean, so it cannot disagree with the actual feature wiring.
`identity` renders a destination back to a canonical string for logs and
status, and `default_prefix` supplies `backup` when none was given.
Surfaces:
- Rust API: `service_backup::BackupDestination::from_uri` - parse an operator-supplied destination string.
- Rust API: `service_backup::BackupDestination::identity` - the canonical rendering used in logs and status.
- Rust API: `service_backup::BackupDestination::default_prefix` - the prefix, defaulting to `backup`.
- Rust API: `service_backup::SUPPORTED_SCHEMES` / `SchemeInfo` - the scheme inventory with per-build sink availability.
Rust internal: the bucket/prefix split with its empty-bucket rejection, and the ordering that makes the parse table and the error message share one source.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - each of the three schemes parses a well-formed URI and renders back the expected identity, including `file:///tmp/backups` becoming `local:/tmp/backups`.
- security: `cargo test -p service-backup --lib` - a URI with an empty bucket is rejected rather than accepted with a blank bucket, and an unsupported scheme's error contains every entry in `SUPPORTED_SCHEMES` so the refusal cannot name a stale subset.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Three-scheme parse and identity | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `parses_object_store_uris` asserts the exact identity for all three schemes, so the round trip is proven rather than assumed |
| Empty bucket rejected | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `rejects_missing_bucket` proves `s3:///prefix` and `gs://` both fail, so a typo cannot produce a request against a bucketless URL |
| Error names every accepted scheme | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `supported_schemes_match_from_uri_error_message` iterates the constant and asserts each scheme appears in the failure text, so adding a scheme without updating the message fails the build |
| Every listed scheme actually parses | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `supported_schemes_each_parse_successfully` proves the inventory cannot advertise a scheme the parser rejects |

#### CRD-Safe Scheduled Policy

ID: crd-safe-scheduled-policy
Root WI: 3376
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
The runtime `BackupPolicy` carries a tagged `BackupDestination` enum, and a
tagged enum's schema cannot be embedded in a Kubernetes CRD structural schema.
So the operator-facing shape is deliberately different: `ScheduledBackupPolicy`
is flat — a cron string, a destination *string*, and an optional retention in
seconds — and its generated schema has no `oneOf`. Every service operator
shares that one shape rather than each inventing its own backup stanza.
Crossing from the flat projection to the runtime policy happens through
exactly one method, `to_runtime_policy`, which is also the only validation
point: an empty or whitespace-only schedule is rejected, and the destination
string is run through `BackupDestination::from_uri` so an unparseable
destination fails at admission rather than at 03:00 when the CronJob fires.
`TryFrom<&ScheduledBackupPolicy>` delegates to that same method, so there is no
second conversion path that could skip a check. Serialization is camelCase
throughout, matching Kubernetes convention, and an absent retention is omitted
from the JSON rather than emitted as null.
Surfaces:
- Rust API: `service_backup::ScheduledBackupPolicy` - the flat CRD-embeddable projection.
- Rust API: `service_backup::ScheduledBackupPolicy::to_runtime_policy` - the single validated conversion.
- Rust API: `service_backup::BackupPolicy` - the runtime policy with the tagged destination.
- Rust API: `service_backup::RetentionPolicy` / `RetentionPolicy::max_age_seconds` - age retention, `None` meaning keep everything.
Rust internal: the `TryFrom` delegation that prevents a second unvalidated conversion path.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - the flat policy serializes to exactly the three expected camelCase fields, the generated schema types `destination` as a plain string and carries no `oneOf`, and a valid conversion carries schedule, destination identity and retention through unchanged.
- security: `cargo test -p service-backup --lib` - a whitespace-only schedule and an `ftp://` destination are both rejected by the conversion, so an invalid policy cannot reach a runner.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Structural-schema-safe projection | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `scheduled_policy_is_flat_and_structural_schema_safe` asserts the exact JSON and that the generated schema has no `oneOf`, so the shape a CRD can actually hold is pinned rather than described |
| One validated conversion path | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `scheduled_policy_uses_one_validated_runtime_conversion` proves both the success carry-through and the two rejections, so validation cannot be bypassed by converting a second way |
| camelCase wire shape | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `policy_serializes_camel_case` pins `maxAgeSeconds`, so a rename cannot silently break every existing CR |

#### Retained Object Write

ID: retained-object-write
Root WI: 3376
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
One backup run is one call. `run_backup_once` takes an already-consistent
payload and a timestamp, writes it through a `BackupSink`, then applies the
policy's age retention, and returns what it actually did: the sink identity,
the key written, the payload's byte count, and how many objects retention
removed. A policy with no `max_age_seconds` prunes nothing and reports zero
rather than skipping the field. The key is derived from the timestamp, so
objects sort by time and retention has something to work with; the local sink
names them `<prefix>-<unix-seconds>.json` and the S3 sink
`<prefix>/backup-<unix-seconds>.json`, with a matching parser that recovers the
timestamp from a key and returns nothing for a key it did not write — so
retention on a shared prefix deletes only this crate's own objects. Local
writes go through `storage_durable::atomic_write` with `FsyncPolicy::Always`,
so a crash mid-backup cannot leave a torn object where a whole one is
expected.
Surfaces:
- Rust API: `service_backup::run_backup_once` - write one object and apply retention.
- Rust API: `service_backup::BackupRunResult` / `BackupObject` - what the run produced.
- Rust API: `service_backup::BackupSink` - the `put` / `prune` / `identity` contract.
- Rust API: `service_backup::LocalFsSink` / `LocalFsSink::from_destination` - the always-available local sink.
- Rust API: `service_backup::sink_from_destination` - the destination-to-sink dispatch.
Rust internal: the S3 key builder and its inverse parser, the prefix normalization shared by both, and the atomic durable write under the local sink.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - a run against a local sink reports the exact byte count and unix seconds it was given and the named key exists on disk; a round trip writes an object and a subsequent zero-age prune removes exactly one, leaving the directory empty.
- security: `cargo test -p service-backup --lib --features s3` - the S3 key built for a normalized nested prefix is exactly `nested/prefix/backup-42.json` and the inverse parser recovers `42` from it while returning nothing for `not-a-backup.json`, so retention cannot delete a foreign object that merely shares the prefix.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Run reports what it wrote | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `runner_reports_written_object` asserts byte count, unix seconds and the on-disk existence of the reported key, so the summary is observed rather than constructed |
| Write then retention | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `local_sink_round_trip_and_prune` writes an object and proves a zero-age prune returns 1 and empties the directory, so retention is real rather than a no-op returning a plausible number |
| Key build and inverse parse | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib --features s3`; `key_helpers_normalize_prefixes` pins the built key and proves the parser recovers the timestamp and rejects a non-backup name, so prefix-scoped retention cannot delete a stranger's object |

### Non-Core Features

#### Fail-Loud Unlinked Adapter

ID: fail-loud-unlinked-adapter
Root WI: 3376
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
S3 support is a crate feature, so a runner binary can be built without it. A
policy naming an `s3://` destination still parses in such a build — the schema
is stable regardless of what is linked — but the sink it resolves to is
`UnsupportedCloudSink`, whose `put` and `prune` fail immediately with a message
naming the destination, the missing feature, and the exact remedy
(`--features s3`, or use `file://`). The alternative is the dangerous one: a
build silently falling back to a local path would report a successful backup
while the bytes never reached the bucket the operator asked for. This is why
`SchemeInfo::sink_available` exists and why it is computed with `cfg!` — an
operator can see, per build, which of the parseable schemes can actually
store anything. S3 with a `credentials_secret` is refused the same way: secret
-mounted credentials are not implemented, so the destination is rejected at
sink construction with the secret name in the message rather than quietly
falling back to ambient credentials that may belong to a different account.
Surfaces:
- Rust API: `service_backup::UnsupportedCloudSink` - the fail-loud placeholder.
- Rust API: `service_backup::SchemeInfo::sink_available` - per-build sink availability.
Rust internal: the `cfg`-selected arm of `sink_from_destination`, and the `credentials_secret` rejection in the S3 sink constructor.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - in a build without the `s3` feature, an `s3://` destination still resolves to a sink, and that sink's `put` fails with a message containing both the feature name and the rebuild flag.
- security: `cargo test -p service-backup --lib --features s3` - a destination carrying `credentials_secret` is rejected at construction with the secret name in the error, so an unimplemented credential path cannot silently degrade to ambient credentials.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Missing adapter fails with the remedy | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `s3_sink_reports_feature_action_when_unlinked` asserts the error contains both `` `s3` feature `` and `--features s3`, so an operator gets the fix rather than a silent local write |
| Unsupported credential path refused | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib --features s3`; `credentials_secret_is_explicitly_unsupported` proves the error names the secret, so an unimplemented path cannot be mistaken for a working one |

#### Exact Object Restore Fetch

ID: exact-object-restore-fetch
Root WI: 3376
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Writing and restoring need different URI shapes, and conflating them is how a
restore ends up reading a directory. `BackupDestination` names a *prefix* to
write under; `fetch_backup_object` names one *object* to read, and is
deliberately narrower: `s3://bucket` is rejected because it has no key, and so
is `s3://bucket/` — a bucket with an empty key is not an object. It accepts the
same three schemes as the writer and produces the same style of error for
anything else, listing every supported scheme. This is the surface bootstrap
and restore paths use to pull one named snapshot back.
Surfaces:
- Rust API: `service_backup::fetch_backup_object` - read one exact backup object by URI.
Rust internal: the bucket/key split that rejects an empty key, distinct from the writer's bucket/prefix split which permits one.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - a `file://` object URI returns the exact bytes written at that path.
- security: `cargo test -p service-backup --lib` - `s3://bucket`, `s3:///key` and `s3://bucket/` are all rejected, so a restore cannot be pointed at a prefix and read something arbitrary; and an unsupported scheme's error names every supported scheme rather than a stale subset.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Exact object read | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `fetches_exact_file_object` writes bytes and asserts the fetch returns them verbatim, so the read path is proven end to end |
| Prefix cannot masquerade as an object | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `s3_object_uri_requires_bucket_and_key` rejects all three degenerate forms, so a restore URI missing its key fails loudly instead of resolving to something else |
| Fetch error lists every scheme | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `unsupported_scheme_error_lists_every_supported_scheme` iterates the shared constant, so the read path's error cannot drift from the write path's |

#### Non-Drifting Scheme Documentation

ID: non-drifting-scheme-documentation
Root WI: 3376
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A CLI `llm` topic that hand-copies the accepted scheme list into a static
string is wrong the moment a scheme is added, and nothing fails when it goes
stale. Here the destination section is generated at call time from
`SUPPORTED_SCHEMES` — the same constant `from_uri` parses against and
`sink_from_destination` dispatches on — so a topic body cannot describe a
scheme set the parser does not implement, including the per-build
`sink_available` flag. The topic is offered in both the flat and sectioned
forms `cli-std` expects, and the two agree on identity rather than being two
independently maintained descriptions of the same crate.
Surfaces:
- Rust API: `service_backup::llm::topic` - the flat CLI topic.
- Rust API: `service_backup::llm::sectioned_topic` - the sectioned form with the generated destination section.
Rust internal: the `TopicSection::Generated` rendering that reads the scheme constant at call time.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib` - both topic forms are non-empty, conform to the `cli-std` topic shape, and agree on identity.
- security: `cargo test -p service-backup --lib` - the generated destination section is asserted to contain every entry in `SUPPORTED_SCHEMES`, so documentation that omits a real scheme fails the build rather than misleading an operator.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Generated section tracks the parser | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `sectioned_topic_destination_section_lists_every_supported_scheme` iterates the constant against the rendered text, so the docs cannot outlive the scheme table |
| Both topic forms agree | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib`; `sectioned_topic_matches_static_topic_identity` and `sectioned_topic_conforms` prove the two surfaces are one description rather than two |

#### Authenticated Admin Snapshot Transport

ID: authenticated-admin-snapshot-transport
Root WI: 3376
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Before a runner can write a backup it has to get the bytes, and every service
backup CLI does that the same way: call the service's admin snapshot endpoint
with a bearer token. Rather than each CLI writing its own client, the
`http-client` feature provides the standard one. The token is optional — an
unauthenticated local service is a legitimate case — and when present it is
sent as bearer authorization. A non-success response is not collapsed into a
generic failure: the status and the response body are both carried into the
error, because a backup that failed because the admin endpoint returned 403
should say 403 and not "backup failed".
Surfaces:
- Rust API: `service_backup::fetch_admin_snapshot` - fetch snapshot bytes from an admin endpoint.
- Rust API: `service_backup::run_admin_snapshot_backup` - fetch and then write through a sink in one call.
EC Dimensions:
- behavior: `cargo test -p service-backup --lib --features http-client` - against a mock admin endpoint the fetch returns the exact snapshot bytes and the request carries the expected bearer authorization.
- security: `cargo test -p service-backup --lib --features http-client` - a non-success response produces an error retaining both the HTTP status and the response body, so an authorization failure is diagnosable rather than anonymous.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Bearer-authenticated snapshot fetch | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib --features http-client`; `fetches_exact_snapshot_bytes_with_bearer_auth` runs against a wiremock server asserting both the header and the returned bytes, so authentication is proven on the wire |
| Failure keeps status and body | change | 3376 | implemented | verified | smoke | `cargo test -p service-backup --lib --features http-client`; `keeps_non_success_status_and_body_in_the_diagnostic` proves the error is not collapsed, so an operator sees why the endpoint refused |

## Not Promised Here

The following behavior exists in the crate but is deliberately given no work
root, because no test in this repository executes it. It is described so the
absence is visible rather than implied:

- **The GCS adapter's I/O.** `GcsSink` construction is covered — including
  the `STORAGE_EMULATOR_HOST` selection and prefix normalization — but its
  media upload, download, delete, paginated listing, and metadata-server
  workload-identity token path have no test in this repository. The adapter
  is a `HANDWRITE` block and is exercised only against a live GCS or Vat
  emulator.
- **The S3 adapter's I/O.** Key construction and the inverse parse are
  covered; `put`, `get_object`, and the paginated `prune` are not.
  `integration_uploads_and_prunes_when_env_is_available` returns early unless
  `SERVICE_BACKUP_S3_TEST_BUCKET` is set, so it passes without executing
  anything and is never cited above as a gate.
- **Age retention on the local sink using filesystem mtime.** `prune` compares
  `modified()` against a cutoff rather than parsing the key's timestamp, so
  an object touched after being written survives retention longer than its key
  suggests. Only the zero-age case is tested.
- **`tests/behavior_shared_service_backup_contract.rs`.** It is an `#[ignore]`d
  scaffold that shells out to `cargo test -p service-backup`, so it is not a
  gate and is never named as one.
