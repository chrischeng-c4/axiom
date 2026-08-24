fn classify_handle(id: u64) -> bool {
    id >= HANDLE_MIN_ID
}

fn classify_large_handle(id: u64) -> bool {
    id > 4096
}
