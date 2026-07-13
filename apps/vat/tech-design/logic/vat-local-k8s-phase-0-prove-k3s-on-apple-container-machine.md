---
id: "1539"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-local-k8s-phase-0-feasibility-logic
entry: start
nodes:
  start: { kind: start, label: "begin real-host Apple-container and k3s feasibility spike" }
  isolate: { kind: process, label: "allocate a unique VAT-owned probe namespace and record pre-existing container state" }
  machine: { kind: process, label: "select or build a systemd-capable OCI image and create a persistent Apple container machine" }
  persistence: { kind: decision, label: "root commands and probe state survive machine stop then run" }
  persistence_blocked: { kind: process, label: "record machine-image or persistence contradiction" }
  prerequisites: { kind: process, label: "probe cgroup v2 overlayfs namespaces mount propagation netfilter sysctls containerd and runc" }
  prerequisites_met: { kind: decision, label: "all required k3s kernel and runtime prerequisites are usable" }
  prerequisite_blocked: { kind: process, label: "record missing capability and custom-kernel or no-go consequence" }
  bootstrap: { kind: process, label: "install and start single-node k3s inside the persistent machine" }
  control_plane: { kind: decision, label: "node Ready and CoreDNS CNI metrics Traefik service load balancer and local-path provisioner are healthy" }
  control_plane_blocked: { kind: process, label: "record k3s bootstrap or addon failure with logs" }
  host_api: { kind: process, label: "export an isolated host kubeconfig and reconcile API endpoint and TLS SAN across restart" }
  host_api_ready: { kind: decision, label: "macOS kubectl reaches the API before and after machine restart" }
  host_api_blocked: { kind: process, label: "record endpoint TLS or address-reconciliation blocker" }
  workload_matrix: { kind: process, label: "apply Deployment Job StatefulSet ConfigMap Secret Service and Ingress fixtures" }
  developer_network: { kind: decision, label: "host port-forward NodePort and Ingress complete application-level round trips without false readiness" }
  network_blocked: { kind: process, label: "record published-port and forwarding constraint including WI 1526 interaction" }
  local_image: { kind: decision, label: "vat-built OCI image reaches k3s containerd without public registry push and pod reports its digest" }
  image_blocked: { kind: process, label: "record image-store transfer contradiction" }
  storage: { kind: decision, label: "PVC survives pod replacement and machine restart then is reclaimable on delete" }
  storage_blocked: { kind: process, label: "record persistent-volume constraint" }
  multi_node: { kind: process, label: "probe a second Apple machine server-agent join plus cross-node CNI DNS and Service routing" }
  evidence: { kind: process, label: "record cold create warm restart idle RSS disk and all command outputs" }
  cleanup: { kind: process, label: "delete every VAT-owned probe machine network workload image and forwarder" }
  verdict: { kind: decision, label: "single-node path satisfies every required Phase 1 prerequisite" }
  go: { kind: terminal, label: "GO: publish evidence and unlock microvm-k3s single-node backend design" }
  conditional: { kind: terminal, label: "CONDITIONAL GO: single-node works but explicitly retain measured bounded blockers for a later child WI" }
  no_go: { kind: terminal, label: "NO-GO: do not author backend implementation until the recorded substrate contradiction is resolved" }
edges:
  - { from: start, to: isolate }
  - { from: isolate, to: machine }
  - { from: machine, to: persistence }
  - { from: persistence, to: prerequisites, label: "persists" }
  - { from: persistence, to: persistence_blocked, label: "does not persist" }
  - { from: persistence_blocked, to: cleanup }
  - { from: prerequisites, to: prerequisites_met }
  - { from: prerequisites_met, to: bootstrap, label: "usable" }
  - { from: prerequisites_met, to: prerequisite_blocked, label: "missing" }
  - { from: prerequisite_blocked, to: cleanup }
  - { from: bootstrap, to: control_plane }
  - { from: control_plane, to: host_api, label: "healthy" }
  - { from: control_plane, to: control_plane_blocked, label: "unhealthy" }
  - { from: control_plane_blocked, to: cleanup }
  - { from: host_api, to: host_api_ready }
  - { from: host_api_ready, to: workload_matrix, label: "reachable" }
  - { from: host_api_ready, to: host_api_blocked, label: "unreachable" }
  - { from: host_api_blocked, to: cleanup }
  - { from: workload_matrix, to: developer_network }
  - { from: developer_network, to: local_image, label: "round trips pass" }
  - { from: developer_network, to: network_blocked, label: "host path blocked" }
  - { from: network_blocked, to: cleanup }
  - { from: local_image, to: storage, label: "digest verified" }
  - { from: local_image, to: image_blocked, label: "transfer blocked" }
  - { from: image_blocked, to: cleanup }
  - { from: storage, to: multi_node, label: "persists" }
  - { from: storage, to: storage_blocked, label: "does not persist" }
  - { from: storage_blocked, to: cleanup }
  - { from: multi_node, to: evidence }
  - { from: evidence, to: cleanup }
  - { from: cleanup, to: verdict }
  - { from: verdict, to: go, label: "all P1 prerequisites pass" }
  - { from: verdict, to: conditional, label: "only deferred multi-node limitation remains" }
  - { from: verdict, to: no_go, label: "any required single-node prerequisite failed" }
---
flowchart TD
    start([begin real-host feasibility spike]) --> isolate[allocate VAT-owned probe namespace]
    isolate --> machine[create persistent Apple machine]
    machine --> persistence{state survives stop/run}
    persistence -- persists --> prerequisites[probe k3s prerequisites]
    persistence -- fails --> persistence_blocked[record persistence contradiction]
    prerequisites --> prerequisites_met{k3s prerequisites usable}
    prerequisites_met -- usable --> bootstrap[start single-node k3s]
    prerequisites_met -- missing --> prerequisite_blocked[record kernel/runtime blocker]
    bootstrap --> control_plane{node and bundled addons healthy}
    control_plane -- healthy --> host_api[export and reconcile host kubeconfig]
    control_plane -- unhealthy --> control_plane_blocked[record bootstrap blocker]
    host_api --> host_api_ready{macOS kubectl works after restart}
    host_api_ready -- reachable --> workload_matrix[apply workload matrix]
    host_api_ready -- unreachable --> host_api_blocked[record API TLS/endpoint blocker]
    workload_matrix --> developer_network{host network round trips pass}
    developer_network -- pass --> local_image{local OCI image digest verified}
    developer_network -- blocked --> network_blocked[record forwarding blocker]
    local_image -- yes --> storage{PVC survives restart}
    local_image -- no --> image_blocked[record image transfer blocker]
    storage -- yes --> multi_node[probe optional multi-node path]
    storage -- no --> storage_blocked[record storage blocker]
    multi_node --> evidence[record timing and footprint evidence]
    persistence_blocked --> cleanup[delete all VAT-owned probe resources]
    prerequisite_blocked --> cleanup
    control_plane_blocked --> cleanup
    host_api_blocked --> cleanup
    network_blocked --> cleanup
    image_blocked --> cleanup
    storage_blocked --> cleanup
    evidence --> cleanup
    cleanup --> verdict{all required P1 prerequisites pass}
    verdict -- yes --> go([GO unlock Phase 1])
    verdict -- only multi-node deferred --> conditional([CONDITIONAL GO])
    verdict -- no --> no_go([NO-GO])
```
