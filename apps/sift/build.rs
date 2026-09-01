// HANDWRITE-BEGIN gap="sift-build-provenance" tracker="1604" reason="Stamp Sift build provenance from the shared build-stamp crate."
//! Stamp Sift build provenance so standard CLI operations can report the
//! exact source revision, build time, and target triple.

fn main() {
    build_stamp::stamp("SIFT");
}
// HANDWRITE-END
