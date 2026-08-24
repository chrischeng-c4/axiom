fn duplicate_iterator_handles(id: u64) {
    let first_handle = MbValue::from_int(id as i64);
    let second_handle = MbValue::from_int(id as i64);
    let _ = (first_handle, second_handle);
}
