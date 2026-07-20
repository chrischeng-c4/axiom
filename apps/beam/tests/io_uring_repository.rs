// HANDWRITE-BEGIN gap="missing-generator:unit-test:5750bcb4" tracker="#2151" reason="scaffold for apps/beam/tests/io_uring_repository.rs — fill in by hand and update tracker when codegen is ready"
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use beam::domain::ports::{VectorRepository, RepositoryError};
use beam::infrastructure::io_uring_repo::IoUringVectorRepository;

fn get_temp_file(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("beam_test_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

#[tokio::test]
async fn test_missing_file_error() {
    let path = get_temp_file("missing");
    let repo = IoUringVectorRepository::new(path.clone());
    assert!(repo.is_err());
    let err = repo.err().unwrap().to_string();
    assert!(err.contains("Storage file is missing"));
}

#[tokio::test]
async fn test_invalid_alignment_error() {
    let path = get_temp_file("alignment");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0u8; 16]).unwrap();
    }

    let repo = IoUringVectorRepository::new(path.clone()).unwrap();
    // offset 3 is not aligned to 4 (alignment check)
    let res = repo.fetch_async(&[3], 4).await;
    assert!(res.is_err());
    let err = res.err().unwrap();
    let typed_err = err.downcast_ref::<RepositoryError>().unwrap();
    match typed_err {
        RepositoryError::InvalidAlignment { offset, alignment } => {
            assert_eq!(*offset, 3);
            assert_eq!(*alignment, 4);
        }
        _ => panic!("Expected InvalidAlignment"),
    }
}

#[tokio::test]
async fn test_out_of_range_error() {
    let path = get_temp_file("out_of_range");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0u8; 8]).unwrap();
    }

    let repo = IoUringVectorRepository::new(path.clone()).unwrap();
    // offset 8 + size 4 = 12 > 8
    let res = repo.fetch_async(&[8], 4).await;
    assert!(res.is_err());
    let err = res.err().unwrap();
    let typed_err = err.downcast_ref::<RepositoryError>().unwrap();
    match typed_err {
        RepositoryError::OutOfRange { offset, size, file_size } => {
            assert_eq!(*offset, 8);
            assert_eq!(*size, 4);
            assert_eq!(*file_size, 8);
        }
        _ => panic!("Expected OutOfRange"),
    }
}

#[tokio::test]
async fn test_concurrent_fetches() {
    let path = get_temp_file("concurrent");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        // Write 4 vectors of 4 bytes each (total 16 bytes)
        let data = [
            1u8, 2, 3, 4,
            5, 6, 7, 8,
            9, 10, 11, 12,
            13, 14, 15, 16,
        ];
        f.write_all(&data).unwrap();
    }

    let repo = Arc::new(IoUringVectorRepository::new(path.clone()).unwrap());

    // Spawn multiple concurrent reads
    let mut handles = vec![];
    for i in 0..10 {
        let repo_clone = Arc::clone(&repo);
        handles.push(tokio::spawn(async move {
            let res = repo_clone.fetch_async(&[0, 8], 4).await.unwrap();
            assert_eq!(res, vec![1, 2, 3, 4, 9, 10, 11, 12]);
            let res2 = repo_clone.fetch_async(&[4, 12], 4).await.unwrap();
            assert_eq!(res2, vec![5, 6, 7, 8, 13, 14, 15, 16]);
            i
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}
// HANDWRITE-END
