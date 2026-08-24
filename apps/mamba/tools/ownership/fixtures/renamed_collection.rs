fn renamed_collection(item: MbValue) -> *mut MbObject {
    let original = vec![item];
    let renamed = original;
    MbObject::new_list(renamed)
}
