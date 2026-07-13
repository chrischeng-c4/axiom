---
id: "1539"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-local-k8s-phase-0-probe-contract
entry: begin
nodes:
  begin: { kind: start, label: "run the real-host Phase 0 probe from a unique VAT-owned temporary root" }
  host_tools: { kind: process, label: "record container kubectl and host versions plus all pre-existing container resources" }
  machine_create: { kind: process, label: "create one persistent systemd-capable Apple machine with explicit CPU memory and no user home mount" }
  persistence_check: { kind: decision, label: "root command marker and machine filesystem state survive stop then run" }
  kernel_matrix: { kind: process, label: "collect cgroup v2 overlayfs namespace mount propagation nft or iptables br_netfilter sysctl containerd and runc evidence inside the machine" }
  kernel_ok: { kind: decision, label: "the observed substrate meets k3s server and workload runtime requirements without hidden Docker dependency" }
  k3s_bootstrap: { kind: process, label: "install and start single-node k3s then retain server and agent logs under the probe root" }
  ready_matrix: { kind: decision, label: "node Ready plus CoreDNS CNI metrics Traefik service load balancer and local-path provisioner reach their expected states before timeout" }
  kubeconfig_export: { kind: process, label: "copy a VAT-owned kubeconfig to macOS and reconcile server endpoint and TLS SAN using the observed machine route" }
  host_api: { kind: decision, label: "host kubectl reads the cluster both before and after a machine restart" }
  workload_matrix: { kind: process, label: "apply Deployment Job StatefulSet ConfigMap Secret ClusterIP Service and Ingress fixtures" }
  service_paths: { kind: decision, label: "port-forward NodePort and Ingress complete application HTTP round trips from macOS and no unreachable published port is reported Ready" }
  image_path: { kind: decision, label: "a locally built OCI image is loaded into k3s containerd without public push and a running pod reports the same digest" }
  volume_path: { kind: decision, label: "a StatefulSet PVC retains its sentinel through pod replacement and machine stop/run then is removed by explicit teardown" }
  multi_node_probe: { kind: process, label: "attempt one server plus one agent machine then record join CNI DNS and Service routing result as P2 evidence" }
  measure: { kind: process, label: "record cold create warm restart idle RSS disk use and every pass fail command output" }
  cleanup: { kind: process, label: "delete workloads k3s machines networks image artifacts and forwarding processes regardless of result" }
  classify: { kind: decision, label: "all mandatory single-node ACs pass and cleanup evidence is complete" }
  go: { kind: terminal, label: "GO: attach evidence and permit Phase 1 microvm-k3s backend TD" }
  conditional_go: { kind: terminal, label: "CONDITIONAL GO: only explicitly scoped P2 multi-node limitation remains" }
  no_go: { kind: terminal, label: "NO-GO: record contradiction and keep Phase 1 implementation blocked" }
edges:
  - { from: begin, to: host_tools }
  - { from: host_tools, to: machine_create }
  - { from: machine_create, to: persistence_check }
  - { from: persistence_check, to: kernel_matrix, label: "pass" }
  - { from: persistence_check, to: cleanup, label: "fail" }
  - { from: kernel_matrix, to: kernel_ok }
  - { from: kernel_ok, to: k3s_bootstrap, label: "pass" }
  - { from: kernel_ok, to: cleanup, label: "fail" }
  - { from: k3s_bootstrap, to: ready_matrix }
  - { from: ready_matrix, to: kubeconfig_export, label: "pass" }
  - { from: ready_matrix, to: cleanup, label: "fail" }
  - { from: kubeconfig_export, to: host_api }
  - { from: host_api, to: workload_matrix, label: "pass" }
  - { from: host_api, to: cleanup, label: "fail" }
  - { from: workload_matrix, to: service_paths }
  - { from: service_paths, to: image_path, label: "pass" }
  - { from: service_paths, to: cleanup, label: "fail" }
  - { from: image_path, to: volume_path, label: "pass" }
  - { from: image_path, to: cleanup, label: "fail" }
  - { from: volume_path, to: multi_node_probe, label: "pass" }
  - { from: volume_path, to: cleanup, label: "fail" }
  - { from: multi_node_probe, to: measure }
  - { from: measure, to: cleanup }
  - { from: cleanup, to: classify }
  - { from: classify, to: go, label: "all single-node ACs pass" }
  - { from: classify, to: conditional_go, label: "only multi-node remains deferred" }
  - { from: classify, to: no_go, label: "any mandatory AC fails" }
---
flowchart TD
    begin([start isolated Phase 0 probe]) --> host_tools[record host tools and state]
    host_tools --> machine_create[create persistent Apple machine]
    machine_create --> persistence_check{state persists across stop/run}
    persistence_check -- pass --> kernel_matrix[collect kernel runtime matrix]
    persistence_check -- fail --> cleanup[cleanup all owned resources]
    kernel_matrix --> kernel_ok{k3s prerequisites usable}
    kernel_ok -- pass --> k3s_bootstrap[start k3s]
    kernel_ok -- fail --> cleanup
    k3s_bootstrap --> ready_matrix{node and bundled addons ready}
    ready_matrix -- pass --> kubeconfig_export[export host kubeconfig]
    ready_matrix -- fail --> cleanup
    kubeconfig_export --> host_api{host kubectl survives restart}
    host_api -- pass --> workload_matrix[apply workload fixtures]
    host_api -- fail --> cleanup
    workload_matrix --> service_paths{host app round trips}
    service_paths -- pass --> image_path{local image digest matches}
    service_paths -- fail --> cleanup
    image_path -- pass --> volume_path{PVC survives restart}
    image_path -- fail --> cleanup
    volume_path -- pass --> multi_node_probe[probe optional multi-node]
    volume_path -- fail --> cleanup
    multi_node_probe --> measure[record timing and footprint]
    measure --> cleanup
    cleanup --> classify{mandatory single-node ACs pass}
    classify -- yes --> go([GO])
    classify -- P2 only --> conditional_go([CONDITIONAL GO])
    classify -- no --> no_go([NO-GO])
```
