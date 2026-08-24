fn exact_suffix(buf: MbList, values: Vec<MbValue>) {
    MbObject::new_list_inline(buf);
    MbObject::new_list_inline_untracked(buf);
    MbObject::new_list_untracked(values);
    MbObject::new_list_inline_extra(buf);
}
