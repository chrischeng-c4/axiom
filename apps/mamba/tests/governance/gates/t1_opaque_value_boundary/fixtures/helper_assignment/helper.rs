fn make_handle(id: u64) -> MbValue {
    MbValue::from_int(id as i64)
}

fn assign_iterator_handle(id: u64) {
    let iterator_handle = make_handle(id);
    let _ = iterator_handle;
}
