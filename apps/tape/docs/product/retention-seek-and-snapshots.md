# Retention, seek, and snapshots

How long a topic keeps messages, how a subscription moves to a point in the
past, and what a snapshot is. This area is the README capability
`topic-retention`; the legacy journal routes below are the readback surface
the subscription model was built on, and the seek outcome is where they leave.

## Topic retention

- Problem: none open as shipped; the limits below belong to the seek outcome.
- Who: operators bounding disk; subscribers that must not lose unread
  messages.
- Promise: `PUT /topics/{topic}/retention` stores `min_offset`,
  `max_age_seconds`, and `protected_consumers`, and `GET` reads the policy
  back. The retention floor never passes a protected consumer's position.
  Messages stay readable until the floor passes them whether or not any
  subscription acked them. A backfill append lands behind the live head
  without moving any consumer.
- Limits today: `max_age_seconds` is stored but no gate proves expiry by
  age; there is no per-subscription retain-acked toggle; seek is a journal
  read (`?from_offset=`, `?from_timestamp_ms=`), not a subscription
  operation.
- Non-goals: export of retained messages to another store.
- Neighbours: none; first section of the area.
- Status rows: `message-retention-duration`, `retain-acked-messages`,
  `seek-to-offset-or-timestamp`.

## Legacy replay and checkpoint routes

- Problem: none open; this section records a surface that is public today and
  leaving, so no new caller builds on it.
- Who: the acceptance scripts, which read back through these routes;
  operators inspecting a journal offline.
- Promise, for now: `GET /topics/{topic}/replay`,
  `GET /topics/{topic}/replay/stream`, and `GET` and
  `PUT /topics/{topic}/consumers/{consumer}/checkpoint` read the journal and
  move a cursor without a subscription resource; `tape replay` and
  `tape checkpoint` do the same against a `--store` file.
- Non-goals: these routes gain no feature. They are not the seek API.
- Neighbours: Named pull subscriptions in
  [subscriptions.md](subscriptions.md) is built on them; the next section
  retires them.
- Status rows: `legacy-replay-and-checkpoint-routes`.

## Seek, snapshot, and retention (Milestone #119)

- Problem: a subscription cannot be moved to a time, an offset, or a saved
  position; age-based retention is unproven; and the journal is reachable
  through two consumption models a Pub/Sub client never has to learn.
- Who: subscribers replaying after a bug or a deploy; operators bounding a
  topic by age.
- Promise: a subscription can seek to a timestamp, to an offset, or to a
  named snapshot. Snapshots are resources with create, list, and delete.
  Retention by age is proven, with the protected floor kept. The public
  retention shape narrows to a duration; the offset floor and protected
  consumers become internal. The legacy replay, streaming replay, checkpoint,
  and backfill surfaces leave the public API and answer 404, the offline
  `tape replay` and `tape checkpoint` verbs retire, and the route inventory,
  the committed OpenAPI snapshot, and the generated clients agree.
- Non-goals: seek on a topic rather than a subscription; export
  subscriptions; keeping the legacy routes as aliases.
- Open: the name that separates a subscription snapshot from the
  whole-journal backup (a decision record settles it); the default and
  maximum retention duration; whether a snapshot pins retention.
- Neighbours: supersedes Legacy replay and checkpoint routes; narrows Topic
  retention; depends on [subscriptions.md](subscriptions.md) § Subscription
  ack and competing subscribers, because seek moves a subscription's lease
  state, and the acceptance scripts must read back through pull before the
  legacy routes go.
- Outcome: `seek-snapshot-and-retention`. Tracking: [Milestone #119](https://github.com/chrischeng-c4/axiom/milestone/119)

## Non-goals in this area

- `export-subscriptions`: the whole-journal backup in
  [operations.md](operations.md) is disaster recovery, not an export path.
