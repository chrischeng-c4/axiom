(.items | length == 1)
and .items[0].metadata.name == ($instance + "-store-google-apis")
and .items[0].metadata.namespace == $namespace
and .items[0].metadata.labels["app.kubernetes.io/name"] == "sift"
and .items[0].metadata.labels["app.kubernetes.io/instance"] == $instance
and .items[0].metadata.labels["app.kubernetes.io/component"] == "store"
and .items[0].spec.podSelector.matchLabels == {
  "app.kubernetes.io/name":"sift",
  "app.kubernetes.io/instance":$instance,
  "app.kubernetes.io/component":"store",
  "sift.axiom.dev/role":"store"
}
and .items[0].spec.egress == [{
  matches:[{name:"storage.googleapis.com"}],
  ports:[{port:443,protocol:"TCP"}]
}]
