//! Reproducible evidence for the `ln(concurrency)` heuristic.
//!
//! Spins a local hyper server that speaks both HTTP/1.1 and h2c, then sweeps
//! h2c connection counts at several concurrency levels and prints throughput.
//! The `recommended_h2c_connections` value is marked so you can see it land at
//! the saturation knee.
//!
//! Run: `cargo run -p h2c --example conn_sweep --release`
use bytes::Bytes;
use h2c::{recommended_h2c_connections_for, H2cPool};
use http_body_util::Full;
use hyper::service::service_fn;
use hyper::{Request, Response};
use std::convert::Infallible;
use std::time::Instant;

const BODY: &[u8] = br#"{"version":"0.4.3","ok":true}"#;

async fn handle(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from_static(BODY))))
}

async fn load(pool: &H2cPool, base: &str, concurrency: usize, total: usize) -> f64 {
    let per = total / concurrency;
    let t = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..concurrency {
        let pool = pool.clone();
        let base = base.to_string();
        handles.push(tokio::spawn(async move {
            for _ in 0..per {
                let _ = pool.get(&base).send().await.unwrap().bytes().await.unwrap();
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    (per * concurrency) as f64 / t.elapsed().as_secs_f64()
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let io = hyper_util::rt::TokioIo::new(stream);
                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(io, service_fn(handle))
                .await;
            });
        }
    });
    let base = format!("http://{addr}");
    let cores = h2c::cpu_parallelism();
    let total = 200_000usize;
    println!("cores={cores}, {total} requests per cell\n");

    let grid = [1usize, 2, 4, 8, 16];
    print!("conc \\ h2 conns ");
    for k in grid {
        print!("  ×{k:<2}  ");
    }
    println!("   recommended");
    println!("{}", "-".repeat(64));

    for &conc in &[16usize, 64, 256, 1024] {
        print!("{conc:5}          ");
        for k in grid {
            let pool = H2cPool::with_connections(k).unwrap();
            // warm
            for _ in 0..k.max(1) {
                let _ = pool.get(&base).send().await.unwrap().bytes().await.unwrap();
            }
            let rps = load(&pool, &base, conc, total).await;
            print!(" {:5.0}k", rps / 1000.0);
        }
        println!("    {} conns", recommended_h2c_connections_for(conc, cores));
    }
}
