fn nested_commas(a: MbValue, b: MbValue, c: MbValue) -> *mut MbObject {
    MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_list(vec![a, b])),
        MbValue::from_ptr(MbObject::new_list(vec![c])),
    ])
}
