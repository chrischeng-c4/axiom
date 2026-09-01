.roleRef == {
  apiGroup:"rbac.authorization.k8s.io",
  kind:"ClusterRole",
  name:"system:auth-delegator"
}
and .subjects == [
  {kind:"ServiceAccount", name:$instance, namespace:$namespace},
  {kind:"ServiceAccount", name:($instance + "-store"), namespace:$namespace}
]
and .metadata.labels["app.kubernetes.io/name"] == "sift"
and .metadata.labels["app.kubernetes.io/instance"] == $instance
and .metadata.labels["sift.axiom.dev/owner-namespace"] == $namespace
and (.metadata.labels["service-k8s.axiom.dev/owner-uid"] | length > 0)
