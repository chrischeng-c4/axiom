fn first_live_iterator_handle(registry: &Registry) {
    let first = registry.keys().next();
    let _ = first;
}
