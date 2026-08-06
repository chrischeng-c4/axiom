from __future__ import annotations

from dataclasses import dataclass

from raft_core.application.timing import ElectionClock, election_timeout_for
from raft_core.domain.commit_rule import highest_committed
from raft_core.domain.election_rules import is_up_to_date, vote_granted
from raft_core.domain.entry import LogEntry
from raft_core.domain.ids import Index, NodeId, Role, Term
from raft_core.domain.log_view import LogView, backoff_hint, prev_entry_matches
from raft_core.domain.membership import Membership, majority
from raft_core.infrastructure.messages import (
    AppendReq,
    AppendResp,
    InstallSnapshotReq,
    InstallSnapshotResp,
    Outgoing,
    RaftMsg,
    VoteReq,
    VoteResp,
)
from raft_core.infrastructure.persistence import PersistedState
from raft_core.infrastructure.transport import Outbox


@dataclass
class RaftNode:
    node_id: NodeId
    voters: tuple[NodeId, ...]
    peers: tuple[NodeId, ...]  # every other member, voters + learners, sorted
    is_voter: bool
    role: Role
    current_term: Term
    voted_for: NodeId | None
    log: list[LogEntry]  # log[0] has index snapshot_index + 1
    commit_index: Index
    last_applied: Index
    snapshot_index: Index
    snapshot_term: Term
    snapshot: bytes
    installed_snapshot: bytes | None
    next_index: dict[NodeId, Index]
    match_index: dict[NodeId, Index]
    votes: set[NodeId]
    clock: ElectionClock
    leader_id: NodeId | None
    outbox: Outbox

    def view(self) -> LogView:
        return LogView(self.snapshot_index, self.snapshot_term, tuple(self.log))

    def last_index(self) -> Index:
        return self.view().last_index()

    def last_term(self) -> Term:
        return self.view().last_term()

    def term_at(self, index: Index) -> Term:
        return self.view().term_at(index)

    def is_leader(self) -> bool:
        return self.role == Role.LEADER

    def majority_size(self) -> int:
        return majority(len(self.voters))

    def persisted(self) -> PersistedState:
        return PersistedState(
            term=self.current_term,
            voted_for=self.voted_for,
            log=tuple(self.log),
            commit_index=self.commit_index,
            snapshot_index=self.snapshot_index,
            snapshot_term=self.snapshot_term,
            snapshot=self.snapshot,
        )

    def take_outgoing(self) -> tuple[Outgoing, ...]:
        return self.outbox.drain()

    def take_committed(self) -> tuple[LogEntry, ...]:
        out: list[LogEntry] = []
        while self.last_applied < self.commit_index:
            idx = self.last_applied + 1
            entry = self.view().entry_at(idx)
            if entry is not None:
                out.append(entry)
            self.last_applied = idx
        return tuple(out)

    def take_installed_snapshot(self) -> bytes | None:
        got = self.installed_snapshot
        self.installed_snapshot = None
        return got

    def _send(self, to: NodeId, msg: RaftMsg) -> None:
        self.outbox.send(to, msg)

    def compact(self, up_to: Index, snapshot: bytes) -> None:
        if up_to <= self.snapshot_index or up_to > self.last_applied:
            return
        term = self.term_at(up_to)
        drop = up_to - self.snapshot_index
        self.log = self.log[min(drop, len(self.log)) :]
        self.snapshot_index = up_to
        self.snapshot_term = term
        self.snapshot = snapshot

    def tick(self) -> None:
        self.clock.tick()
        if self.role == Role.LEADER:
            if self.clock.heartbeat_due():
                self.clock.reset_heartbeat()
                self._broadcast_append()
        elif self.is_voter and self.clock.election_due():
            self._start_election()

    def _start_election(self) -> None:
        self.current_term += 1
        self.role = Role.CANDIDATE
        self.voted_for = self.node_id
        self.leader_id = None
        self.votes = {self.node_id}
        self.clock.reset_election()
        lli, llt = self.last_index(), self.last_term()
        for v in self.voters:
            if v != self.node_id:
                self._send(
                    v,
                    VoteReq(
                        term=self.current_term,
                        candidate=self.node_id,
                        last_log_index=lli,
                        last_log_term=llt,
                    ),
                )
        self._maybe_become_leader()

    def _maybe_become_leader(self) -> None:
        if self.role != Role.CANDIDATE:
            return
        granted = len([v for v in self.votes if v in self.voters])
        if granted >= self.majority_size():
            self._become_leader()

    def _become_leader(self) -> None:
        self.role = Role.LEADER
        self.leader_id = self.node_id
        nxt = self.last_index() + 1
        self.next_index = {p: nxt for p in self.peers}
        self.match_index = {p: 0 for p in self.peers}
        self.clock.reset_heartbeat()
        self._broadcast_append()

    def _step_down(self, term: Term) -> None:
        if term > self.current_term:
            self.current_term = term
            self.voted_for = None
        self.role = Role.FOLLOWER
        self.clock.reset_election()

    def _broadcast_append(self) -> None:
        for p in self.peers:
            self._send_append_to(p)

    def _send_append_to(self, peer: NodeId) -> None:
        nxt = self.next_index.get(peer, self.last_index() + 1)
        if nxt <= self.snapshot_index:
            self._send(
                peer,
                InstallSnapshotReq(
                    term=self.current_term,
                    leader=self.node_id,
                    snapshot_index=self.snapshot_index,
                    snapshot_term=self.snapshot_term,
                    data=self.snapshot,
                ),
            )
            return
        prev_index = max(nxt - 1, 0)
        prev_term = self.term_at(prev_index)
        entries = tuple(e for e in self.log if e.index >= nxt)
        self._send(
            peer,
            AppendReq(
                term=self.current_term,
                leader=self.node_id,
                prev_log_index=prev_index,
                prev_log_term=prev_term,
                entries=entries,
                leader_commit=self.commit_index,
            ),
        )

    def propose(self, command: bytes) -> Index | None:
        if self.role != Role.LEADER:
            return None
        index = self.last_index() + 1
        self.log.append(
            LogEntry(term=self.current_term, index=index, command=command)
        )
        self._broadcast_append()
        self._maybe_commit()
        return index

    def handle(self, sender: NodeId, msg: RaftMsg) -> None:
        if isinstance(msg, VoteReq):
            self._handle_vote(sender, msg)
        elif isinstance(msg, VoteResp):
            self._handle_vote_resp(sender, msg)
        elif isinstance(msg, AppendReq):
            self._handle_append(msg)
        elif isinstance(msg, AppendResp):
            self._handle_append_resp(sender, msg)
        elif isinstance(msg, InstallSnapshotReq):
            self._handle_install_snapshot(msg)
        elif isinstance(msg, InstallSnapshotResp):
            self._handle_install_snapshot_resp(sender, msg)

    def _handle_vote(self, sender: NodeId, req: VoteReq) -> None:
        if req.term > self.current_term:
            self._step_down(req.term)
        up_to_date = is_up_to_date(
            req.last_log_term,
            req.last_log_index,
            self.last_term(),
            self.last_index(),
        )
        grant = vote_granted(
            req.term,
            self.current_term,
            self.voted_for,
            req.candidate,
            up_to_date,
        )
        if grant:
            self.voted_for = req.candidate
            self.clock.reset_election()
        self._send(sender, VoteResp(term=self.current_term, granted=grant))

    def _handle_vote_resp(self, sender: NodeId, resp: VoteResp) -> None:
        if resp.term > self.current_term:
            self._step_down(resp.term)
            return
        if (
            self.role == Role.CANDIDATE
            and resp.term == self.current_term
            and resp.granted
        ):
            self.votes.add(sender)
            self._maybe_become_leader()

    def _handle_append(self, req: AppendReq) -> None:
        if req.term < self.current_term:
            self._send(
                req.leader,
                AppendResp(
                    term=self.current_term, success=False, match_index=0
                ),
            )
            return
        self._step_down(req.term)
        self.leader_id = req.leader

        if not prev_entry_matches(
            self.view(), req.prev_log_index, req.prev_log_term
        ):
            hint = backoff_hint(self.view(), req.prev_log_index)
            self._send(
                req.leader,
                AppendResp(
                    term=self.current_term, success=False, match_index=hint
                ),
            )
            return

        for e in req.entries:
            if e.index <= self.snapshot_index:
                continue
            pos = e.index - self.snapshot_index - 1
            if pos < len(self.log):
                if self.log[pos].term != e.term:
                    self.log = self.log[:pos]
                    self.log.append(e)
            else:
                self.log.append(e)

        match_index = req.prev_log_index + len(req.entries)
        if req.leader_commit > self.commit_index:
            self.commit_index = min(req.leader_commit, self.last_index())
        self._send(
            req.leader,
            AppendResp(
                term=self.current_term, success=True, match_index=match_index
            ),
        )

    def _handle_append_resp(self, sender: NodeId, resp: AppendResp) -> None:
        if resp.term > self.current_term:
            self._step_down(resp.term)
            return
        if self.role != Role.LEADER or resp.term != self.current_term:
            return

        if resp.success:
            matched = max(self.match_index.get(sender, 0), resp.match_index)
            self.match_index[sender] = matched
            self.next_index[sender] = max(
                self.next_index.get(sender, 1), matched + 1
            )
            old = self.commit_index
            self._maybe_commit()
            if self.commit_index > old:
                self._broadcast_append()
            elif self.next_index.get(sender, 1) <= self.last_index():
                self._send_append_to(sender)
        else:
            if resp.match_index < self.match_index.get(sender, 0):
                return
            n = self.next_index.get(sender, 1)
            n = max(min(max(n - 1, 0), resp.match_index + 1), 1)
            self.next_index[sender] = n
            self._send_append_to(sender)

    def _handle_install_snapshot(self, req: InstallSnapshotReq) -> None:
        if req.term < self.current_term:
            self._send(
                req.leader,
                InstallSnapshotResp(
                    term=self.current_term, snapshot_index=self.snapshot_index
                ),
            )
            return
        self._step_down(req.term)
        self.leader_id = req.leader
        if req.snapshot_index > self.snapshot_index:
            self.log = []
            self.snapshot_index = req.snapshot_index
            self.snapshot_term = req.snapshot_term
            self.snapshot = req.data
            self.installed_snapshot = req.data
            if self.commit_index < req.snapshot_index:
                self.commit_index = req.snapshot_index
            self.last_applied = req.snapshot_index
        self._send(
            req.leader,
            InstallSnapshotResp(
                term=self.current_term, snapshot_index=self.snapshot_index
            ),
        )

    def _handle_install_snapshot_resp(
        self, sender: NodeId, resp: InstallSnapshotResp
    ) -> None:
        if resp.term > self.current_term:
            self._step_down(resp.term)
            return
        if self.role != Role.LEADER or resp.term != self.current_term:
            return
        if resp.snapshot_index > self.match_index.get(sender, 0):
            self.match_index[sender] = resp.snapshot_index
        self.next_index[sender] = resp.snapshot_index + 1
        old = self.commit_index
        self._maybe_commit()
        if self.commit_index > old:
            self._broadcast_append()
        elif self.next_index.get(sender, 1) <= self.last_index():
            self._send_append_to(sender)

    def _maybe_commit(self) -> None:
        if self.role != Role.LEADER:
            return
        self.commit_index = highest_committed(
            self.view(),
            self.voters,
            self.match_index,
            self.node_id,
            self.current_term,
            self.commit_index,
        )


def new_node(node_id: NodeId, membership: Membership) -> RaftNode:
    members = sorted(membership.voters + membership.learners)
    peers = tuple(m for m in members if m != node_id)
    return RaftNode(
        node_id=node_id,
        voters=membership.voters,
        peers=peers,
        is_voter=(node_id in membership.voters),
        role=Role.FOLLOWER,
        current_term=0,
        voted_for=None,
        log=[],
        commit_index=0,
        last_applied=0,
        snapshot_index=0,
        snapshot_term=0,
        snapshot=b"",
        installed_snapshot=None,
        next_index={},
        match_index={},
        votes=set(),
        clock=ElectionClock(election_timeout=election_timeout_for(node_id)),
        leader_id=None,
        outbox=Outbox(),
    )


def from_persisted(
    node_id: NodeId, membership: Membership, state: PersistedState
) -> RaftNode:
    node = new_node(node_id, membership)
    node.current_term = state.term
    node.voted_for = state.voted_for
    node.log = list(state.log)
    node.snapshot_index = state.snapshot_index
    node.snapshot_term = state.snapshot_term
    node.snapshot = state.snapshot
    # clamp: never below the compaction point, never above what we actually hold
    node.commit_index = min(
        max(state.commit_index, node.snapshot_index), node.last_index()
    )
    node.last_applied = node.snapshot_index
    if node.snapshot_index > 0 and node.snapshot != b"":
        node.installed_snapshot = node.snapshot
    return node
