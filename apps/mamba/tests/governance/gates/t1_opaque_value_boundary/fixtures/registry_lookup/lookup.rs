fn extracted_handle(value: MbValue) -> u64 {
    let id = value.as_int().map(|id| id as u64).unwrap_or(0);
    integer_handle_registry::lookup(id);
    id
}

fn consume_iterator_handle(value: MbValue) {
    let id = extracted_handle(value);
    let _ = id;
}
