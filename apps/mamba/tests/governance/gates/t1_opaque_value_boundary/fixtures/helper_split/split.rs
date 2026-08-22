fn produce_iterator_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}

fn consume_iterator_handle(value: MbValue) {
    let id = value.as_int().unwrap_or(0);
    integer_handle_registry::lookup(id);
}
