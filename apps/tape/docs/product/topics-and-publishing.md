# Topics and publishing

A topic is an append-only, offset-numbered journal that exists implicitly on
first use. Publishing is one durable write per request. This area is the
README capability `topic-publish`; the sections after the first are the parts
of Cloud Pub/Sub's message and topic model that tape has not built.

Attributes and ordering keys are published on the message but promised at
delivery, so their sections live in [subscriptions.md](subscriptions.md).

## Durable single publish

- Problem: none open; this is shipped behaviour and the baseline every later
  section extends.
- Who: publishers.
- Promise: `POST /topics/{topic}/append` accepts one JSON message
  `{key?, payload, timestamp_ms?}` and answers with its offset only after the
  write is durable through the WAL group-commit fsync. An oversized body is
  refused with 413 before any write. When the volume is full the node keeps
  serving reads and refuses appends with 507. In a Raft group the append is
  ordered and replicated before the answer, and a follower forwards it to the
  leader. `key` is stored and returned with the event but carries no
  delivery promise yet, and there is no attribute map.
- Non-goals: batch publish, an attribute map, and schema validation are the
  three sections below and in subscriptions; none of them is implied here.
- Neighbours: none; first section of the area.
- Status rows: `publish-single`, `durable-write-path`, `message-attributes`,
  `ordering-key-on-publish`,
  `storage-full-degraded-mode`.

## Resource lifecycle parity (Milestone #123)

- Problem: topics cannot be listed or deleted, a publish carries exactly one
  message, and a subscription cannot expire or be detached from its topic.
- Who: operators managing topics; publishers with high message rates;
  subscribers that want an unused subscription to disappear on its own.
- Promise: explicit create, delete, and list routes for topics; a batch
  publish that reports one offset per message in request order; a
  subscription expiration policy after which the subscription is gone; a
  detached subscription that stays as a resource but receives nothing.
  Implicit topic creation keeps working behind the explicit routes.
- Non-goals: quotas on topic or subscription counts belong to
  [operations.md](operations.md) § Quotas and scale transition. Any item in
  this section the product does not want moves to the ROADMAP non-goals
  rather than staying open.
- Open: whether a batch is durable per message or all-or-nothing; what a
  partial failure answers; the default and maximum expiration duration.
- Neighbours: extends Durable single publish; the expiration and detach
  halves extend [subscriptions.md](subscriptions.md) § Named pull
  subscriptions.
- Outcome: `resource-lifecycle-parity`. Tracking: [Milestone #123](https://github.com/chrischeng-c4/axiom/milestone/123)

## Schema validation (Milestone #122)

- Problem: any JSON value is accepted as `payload`, so a subscriber cannot
  rely on the shape of what it pulls, and a malformed publish is durable
  before anyone notices.
- Who: publishers, who want the refusal before the write; subscribers, who
  want a guaranteed shape.
- Promise: a topic can bind a schema in Avro, Protobuf, or JSON Schema form,
  with revisions; a publish that does not conform is refused with 400 before
  it reaches the WAL and receives no offset; a revision change applies to
  publishes after it only. Schemas have create, list, and delete routes.
- Non-goals: evolution rules beyond pinning a revision; schema-aware filtering
  on subscriptions; validating messages already in the journal.
- Open: the wire encoding of an Avro or Protobuf payload (binary versus JSON
  encoding); whether a topic can change its bound schema in place.
- Neighbours: narrows what Durable single publish accepts as `payload`.
- Outcome: `schema-validation`. Tracking: [Milestone #122](https://github.com/chrischeng-c4/axiom/milestone/122)

## Non-goals in this area

- `pubsub-wire-compatibility`: publishing uses tape's own route and envelope,
  not the Google API shape.
