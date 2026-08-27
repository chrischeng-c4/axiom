# Subscriptions

A subscription is a named cursor into one topic today. In Cloud Pub/Sub it is
a configured resource with per-message acknowledgement, competing
subscribers, push delivery, ordering, and filtering. This area is the README
capability `pull-subscriptions` and owns the largest gap in the STATUS
checklist; four of the seven H1 outcomes land here.

## Named pull subscriptions

- Problem: none open as shipped; the limits below are what the next four
  sections close.
- Who: subscribers; operators reading lag.
- Promise: create, list, show, and delete named subscriptions on a topic over
  HTTP and the offline `--store` CLI; `POST .../subscriptions/{name}/pull`
  with `limit` returns `cursor`, `events`, and `next_offset` from the
  subscription's cursor; `POST .../ack` with `offset` advances the cursor;
  `tape_subscription_lag` reports the distance to the topic head;
  `TAPE_PROVISION_TOPICS` and the custom resource create topics and
  subscriptions at start.
- Limits today: the cursor is cumulative, so acking offset N acks everything
  before it; every puller sees the same window until an ack, so two pullers
  duplicate work rather than share it; there is no ack deadline, redelivery,
  retry policy, dead-letter topic, push config, or filter; grants are per
  topic, so a reader of one subscription can read every subscription on the
  topic; provisioning writes the journal directly instead of proposing through
  Raft.
- Non-goals: nothing beyond the cursor model is promised by this section.
- Neighbours: none; first section of the area.
- Status rows: `subscription-lifecycle-pull`, `pull-sync`, `ack`,
  `declarative-subscription-provisioning`, `per-topic-authorization`,
  `delivery-metrics`.

## Subscription ack and competing subscribers

- Problem: a subscriber cannot acknowledge one message, cannot share a
  subscription across workers, never gets a message back after a crash, and
  has nowhere for a poison message to go.
- Who: subscribers that run more than one worker on one subscription;
  operators watching the oldest unacknowledged message.
- Promise: a subscription is a configured resource with an ack deadline, a
  retry policy of minimum and maximum backoff, and a dead-letter topic with a
  maximum number of delivery attempts. Pull hands out one ackId per message
  under the deadline. A caller acks, extends (`modifyAckDeadline`), or nacks
  each message by ackId. A message whose deadline expires, or that is nacked,
  is redelivered to any puller on the same subscription; after the configured
  attempts it is published to the dead-letter topic instead. The lease table
  and the subscription configuration travel through Raft as replicated
  commands and survive leader failover; declarative provisioning proposes
  through the same path. Grants become subscription-scoped. An
  oldest-unacked-age metric is exposed per subscription.
- Non-goals: push delivery, streaming pull, exactly-once delivery, and
  per-key ordering are their own sections or non-goals.
- Open: the default ack deadline and its bounds; whether the cumulative ack
  route survives as a compatibility alias or the ack body changes shape (the
  existing cursor cases are rewritten against the lease model either way);
  whether the dead-letter topic must pre-exist.
- Neighbours: supersedes the pull-window and ack promise of Named pull
  subscriptions; depends on
  [replication-and-availability.md](replication-and-availability.md)
  § Deterministic failover for the failover proof.
- Outcome: `subscription-ack-and-competing-subscribers`. Tracking: not assigned.

## Push subscriptions

- Problem: a subscriber has to poll; tape cannot call it.
- Who: subscribers exposing an HTTPS endpoint; operators handling the token.
- Promise: a subscription can carry a push configuration with an HTTPS
  endpoint and a bearer token. Tape delivers each message to the endpoint
  itself; a 2xx is the ack; anything else is a failed attempt that backs off
  under the subscription's retry policy and leaves the message in the
  pull-visible in-flight state, so a receiver outage loses nothing. The token
  never appears in logs or metrics.
- Non-goals: Google-signed OIDC tokens, non-HTTPS targets, hand-off to
  `defer`, and push batching.
- Open: the shape of the push request body (Pub/Sub wraps the message and
  the subscription name; tape has its own envelope); whether a push
  subscription can also be pulled.
- Neighbours: extends Subscription ack and competing subscribers and depends
  on its lease and retry policy.
- Outcome: `push-subscriptions`. Tracking: not assigned.

## Ordering keys

- Problem: `key` is stored and returned but delivery ignores it, so two
  messages for one entity can reach two workers in either order.
- Who: publishers that set `key`; subscribers that need per-entity order.
- Promise: messages that share an ordering key are delivered to their holder
  in publish order, including across redelivery and competing pullers;
  messages without a key carry no ordering promise and interleave freely.
- Non-goals: order across more than one shard is
  [replication-and-availability.md](replication-and-availability.md)
  § Multi-shard topology; a public partition API is never offered.
- Open: what happens to later messages on a key while an earlier one is
  in-flight or dead-lettered (hold, or release after dead-letter).
- Neighbours: extends Subscription ack and competing subscribers with holder
  semantics; the publish half is already in
  [topics-and-publishing.md](topics-and-publishing.md) § Durable single
  publish.
- Outcome: `ordering-keys`. Tracking: not assigned.

## Attributes and filters

- Problem: a message carries no metadata beyond `key`, and a subscription
  receives every message on its topic whether it wants it or not.
- Who: publishers that tag messages; subscribers that want a subset.
- Promise: a publish carries a string-to-string attribute map alongside the
  payload, and it round-trips through pull and push. A subscription stores a
  filter expression that is evaluated at delivery; a filtered-out message is
  acked implicitly and does not appear as lag; a filter change applies to
  messages published after it.
- Non-goals: filtering on payload content; schema-aware filtering; `key` is
  the ordering key, not an attribute.
- Open: the filter language (Pub/Sub's attribute syntax or tape's own) and
  whether a filter can be changed on an existing subscription or only set at
  creation.
- Neighbours: extends Named pull subscriptions and Push subscriptions on the
  delivery side, Durable single publish on the publish side.
- Outcome: `attributes-and-filters`. Tracking: not assigned.

## Non-goals in this area

- `streaming-pull`: unary pull and push are the two delivery paths; there is
  no gRPC to stream over.
- `exactly-once-delivery`: at least once, with the offset as the
  deduplication key.
