//! Infrastructure implementation of VectorRepository.

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use crate::domain::ports::{VectorRepository, RepositoryError};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(not(unix))]
trait FileExt {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
}

#[cfg(not(unix))]
impl FileExt for File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::io::{Read, Seek, SeekFrom};
        let mut f = self.try_clone()?;
        f.seek(SeekFrom::Start(offset))?;
        f.read(buf)
    }
}

// <HANDWRITE gap="missing-generator:logic--ffc20b4e" tracker="#2151" reason="logic section in io_uring_repo.rs is hand-written pending codegen support">
/// Infrastructure Adapter implementing VectorRepository using Direct Offset file reading.
pub struct IoUringVectorRepository {
    file: Arc<File>,
    #[cfg(target_os = "linux")]
    linux_backend: Arc<LinuxIoUringBackend>,
}

#[cfg(target_os = "linux")]
struct LinuxIoUringBackend {
    ring: std::sync::Mutex<io_uring::IoUring>,
    file: Arc<File>,
}

#[cfg(target_os = "linux")]
impl LinuxIoUringBackend {
    pub fn new(file: Arc<File>) -> anyhow::Result<Self> {
        let ring = io_uring::IoUring::new(32)?;
        Ok(Self {
            ring: std::sync::Mutex::new(ring),
            file,
        })
    }

    pub fn fetch_async(&self, offsets: &[u64], vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
        use std::os::unix::io::AsRawFd;
        let fd = self.file.as_raw_fd();
        let mut result = Vec::with_capacity(offsets.len() * vector_bytes);
        
        let mut ring = self.ring.lock().unwrap();
        // Split the ring to submit and wait
        let (submitter, mut sq, mut cq) = ring.split();

        for offset in offsets {
            // Validate offset alignment (must be multiple of 4 for f32)
            if offset % 4 != 0 {
                return Err(anyhow::anyhow!(RepositoryError::InvalidAlignment {
                    offset: *offset,
                    alignment: 4,
                }));
            }

            // Validate offset out-of-range
            let file_size = self.file.metadata()?.len();
            if *offset + (vector_bytes as u64) > file_size {
                return Err(anyhow::anyhow!(RepositoryError::OutOfRange {
                    offset: *offset,
                    size: vector_bytes,
                    file_size,
                }));
            }

            let mut buf = vec![0u8; vector_bytes];
            let buf_ptr = buf.as_mut_ptr();
            let entry = io_uring::opcode::Read::new(
                io_uring::types::Fd(fd),
                buf_ptr,
                vector_bytes as u32,
            )
            .offset(*offset)
            .build()
            .user_data(*offset);

            unsafe {
                sq.push(&entry).map_err(|e| anyhow::anyhow!("sq push failed: {e}"))?;
            }
            sq.sync();
            submitter.submit_and_wait(1)?;
            cq.sync();

            let cqe = cq.next().ok_or_else(|| anyhow::anyhow!("cqe missing"))?;
            let res = cqe.result();
            if res < 0 {
                return Err(anyhow::anyhow!(std::io::Error::from_raw_os_error(-res)));
            }
            let read = res as usize;
            if read != vector_bytes {
                return Err(anyhow::anyhow!(RepositoryError::ShortRead {
                    offset: *offset,
                    expected: vector_bytes,
                    got: read,
                }));
            }
            result.extend_from_slice(&buf);
        }

        Ok(result)
    }
}
// </HANDWRITE>

impl IoUringVectorRepository {
    /// Create a new Repository pointing to a physical database storage file.
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!(RepositoryError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            }));
        }
        let file = File::open(&path)?;
        let file = Arc::new(file);

        #[cfg(target_os = "linux")]
        let linux_backend = Arc::new(LinuxIoUringBackend::new(Arc::clone(&file))?);

        Ok(Self {
            file,
            #[cfg(target_os = "linux")]
            linux_backend,
        })
    }
}

impl VectorRepository for IoUringVectorRepository {
    async fn fetch_async(&self, offsets: &[u64], vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
        #[cfg(target_os = "linux")]
        {
            eprintln!("beam: active storage backend = Linux io_uring NVMe repository");
            let backend = Arc::clone(&self.linux_backend);
            let offsets = offsets.to_vec();
            return tokio::task::spawn_blocking(move || {
                backend.fetch_async(&offsets, vector_bytes)
            })
            .await?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("beam: active storage backend = Unix FileExt NVMe fallback");
            let file = Arc::clone(&self.file);
            let offsets = offsets.to_vec();

            tokio::task::spawn_blocking(move || {
                let mut result = Vec::with_capacity(offsets.len() * vector_bytes);
                let mut buf = vec![0u8; vector_bytes];

                let file_size = file.metadata()?.len();

                for offset in offsets {
                    // R2: Treat invalid alignment as typed error (alignment = 4 bytes for floats)
                    if offset % 4 != 0 {
                        return Err(anyhow::anyhow!(RepositoryError::InvalidAlignment {
                            offset,
                            alignment: 4,
                        }));
                    }

                    // R2: Treat out-of-range offsets as typed error
                    if offset + (vector_bytes as u64) > file_size {
                        return Err(anyhow::anyhow!(RepositoryError::OutOfRange {
                            offset,
                            size: vector_bytes,
                            file_size,
                        }));
                    }

                    // Perform read honestly
                    match file.read_at(&mut buf, offset) {
                        Ok(read) if read == vector_bytes => {
                            result.extend_from_slice(&buf);
                        }
                        Ok(read) => {
                            return Err(anyhow::anyhow!(RepositoryError::ShortRead {
                                offset,
                                expected: vector_bytes,
                                got: read,
                            }));
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!(RepositoryError::IoError { source: e }));
                        }
                    }
                }
                Ok(result)
            })
            .await?
        }
    }
}
