fn update_barrier_instance_fields(id: u64) -> MbValue {
    // The enclosing instance exposes this field through generic getattr.
    let barrier_id = id;
    MbValue::from_int(barrier_id as i64)
}

fn get_or_create_barrier(value: MbValue) -> MbValue {
    value.as_int().map(MbValue::from_int).unwrap_or(value)
}
