// HANDWRITE-BEGIN gap="missing-generator:logic:8dbe3ccf" tracker="pending-tracker" reason="New script (mirrors projects/lumen/scripts/dev-cluster.sh): 3-node local raft cluster, sets REPLICAS_PER_SHARD=3/SHARD_COUNT=1/VOTER_COUNT=3/POD_NAME plus TAPE_DATA_DIR/TAPE_PEER_SERVICE/TAPE_PEERS so raft_host::ClusterTopology::from_env resolves peers and TapeRaft::from_topology replicates append/checkpoint-put across 3 tape serve processes on distinct ports."
// TODO: hand-write content for `apps/tape/scripts/dev-cluster.sh`.
// HANDWRITE-END
