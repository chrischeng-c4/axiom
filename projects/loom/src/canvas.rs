//! Workflow canvas / DAG builder (#116, #112).
//!
//! Turns the high-level composition primitives a client expresses — sequential
//! `then` (chain), parallel `fan_out` (group), and `join` (chord callback /
//! fan-in barrier) — into a concrete [`WorkflowRun`] (nodes + deps + stages)
//! that the scheduler drives. Seeded by the canvas shapes in
//! `projects/queue/queue/src/workflow` (chain / group / chord), recast onto
//! loom's server-side DAG model.

use std::collections::BTreeSet;

use crate::model::{Node, NodeId, StageId, TaskSpec, WorkflowRun, WorkflowRunId};

/// Builds a [`WorkflowRun`] from canvas primitives. Each call adds a stage and
/// remembers the "frontier" (the nodes a subsequent `then` depends on).
pub struct WorkflowBuilder {
    run: WorkflowRun,
    counter: usize,
    stage_counter: usize,
    /// Nodes the next sequential `then` should depend on.
    frontier: Vec<NodeId>,
}

impl WorkflowBuilder {
    pub fn new(run_id: impl Into<String>) -> Self {
        Self {
            run: WorkflowRun::new(WorkflowRunId::new(run_id)),
            counter: 0,
            stage_counter: 0,
            frontier: Vec::new(),
        }
    }

    fn next_node_id(&mut self) -> NodeId {
        let id = NodeId::new(format!("n{}", self.counter));
        self.counter += 1;
        id
    }

    fn next_stage_id(&mut self) -> StageId {
        let id = StageId::new(format!("s{}", self.stage_counter));
        self.stage_counter += 1;
        id
    }

    fn add(&mut self, task: TaskSpec, stage: StageId, deps: BTreeSet<NodeId>) -> NodeId {
        let id = self.next_node_id();
        self.run.add_node(Node::new(id.clone(), stage, task, deps));
        id
    }

    /// Chain link: one task that depends on the entire current frontier; it
    /// becomes the new frontier.
    pub fn then(&mut self, task: TaskSpec) -> NodeId {
        let stage = self.next_stage_id();
        let deps: BTreeSet<NodeId> = self.frontier.iter().cloned().collect();
        let id = self.add(task, stage, deps);
        self.frontier = vec![id.clone()];
        id
    }

    /// Group / fan-out: parallel sibling tasks in one stage, each depending on
    /// the current frontier; the whole group becomes the new frontier.
    pub fn fan_out(&mut self, tasks: Vec<TaskSpec>) -> Vec<NodeId> {
        let stage = self.next_stage_id();
        let deps: BTreeSet<NodeId> = self.frontier.iter().cloned().collect();
        let ids: Vec<NodeId> = tasks
            .into_iter()
            .map(|t| self.add(t, stage.clone(), deps.clone()))
            .collect();
        self.frontier = ids.clone();
        id_passthrough(ids)
    }

    /// Chord callback / explicit fan-in: a join task depending on exactly the
    /// given nodes; it becomes the new frontier.
    pub fn join(&mut self, task: TaskSpec, deps: Vec<NodeId>) -> NodeId {
        let stage = self.next_stage_id();
        let id = self.add(task, stage, deps.into_iter().collect());
        self.frontier = vec![id.clone()];
        id
    }

    pub fn build(self) -> WorkflowRun {
        self.run
    }
}

#[inline]
fn id_passthrough(ids: Vec<NodeId>) -> Vec<NodeId> {
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::NodeState;

    fn t(name: &str) -> TaskSpec {
        TaskSpec::new(name)
    }

    #[test]
    fn chain_links_sequentially() {
        let mut b = WorkflowBuilder::new("run-chain");
        let a = b.then(t("a"));
        let c = b.then(t("b"));
        let run = b.build();
        assert!(run.nodes[&a].deps.is_empty());
        assert_eq!(run.nodes[&c].deps.iter().cloned().collect::<Vec<_>>(), vec![a]);
        // only the root is ready.
        assert_eq!(run.ready_nodes().len(), 1);
    }

    #[test]
    fn chord_is_fan_out_then_join() {
        // root -> {x, y, z} (group) -> join (chord callback)
        let mut b = WorkflowBuilder::new("run-chord");
        b.then(t("root"));
        let group = b.fan_out(vec![t("x"), t("y"), t("z")]);
        let join = b.join(t("reduce"), group.clone());
        let run = b.build();

        // the join depends on all three group members (fan-in barrier).
        assert_eq!(run.nodes[&join].deps, group.iter().cloned().collect());
        // the group is one stage of three siblings.
        let group_stage = &run.nodes[&group[0]].stage;
        assert_eq!(run.stages[group_stage].members.len(), 3);
        // join is Pending until the whole group is Done.
        assert_eq!(run.nodes[&join].state, NodeState::Pending);
    }

    #[test]
    fn built_run_drives_to_completion() {
        let mut b = WorkflowBuilder::new("run-x");
        b.then(t("root"));
        let group = b.fan_out(vec![t("x"), t("y")]);
        let join = b.join(t("reduce"), group.clone());
        let mut run = b.build();

        // walk the DAG: root -> group -> join.
        let root = run.ready_nodes();
        assert_eq!(root.len(), 1);
        run.mark_done(&root[0], None);
        let mut g = run.ready_nodes();
        g.sort();
        assert_eq!(g, group);
        for id in &group {
            run.mark_done(id, None);
        }
        assert_eq!(run.ready_nodes(), vec![join.clone()]);
        run.mark_done(&join, None);
        assert_eq!(run.status, crate::model::RunStatus::Succeeded);
    }
}
