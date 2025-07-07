use std::sync::Arc;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use icn_common::{compute_merkle_cid, DagBlock, Did};
use tempfile::tempdir;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

const NUM_BLOCKS: usize = 1000;

fn create_block(id: usize) -> DagBlock {
    let data = format!("data {id}").into_bytes();
    let ts = 0u64;
    let author = Did::new("key", "bench");
    let sig = None;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &sig, &None);
    DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: author,
        signature: sig,
        scope: None,
    }
}

#[cfg(feature = "persist-sled")]
async fn run_sled() -> Duration {
    use icn_dag::sled_store::SledDagStore;
    use icn_dag::StorageService;
    let dir = tempdir().unwrap();
    let store = Arc::new(Mutex::new(
        SledDagStore::new(dir.path().to_path_buf()).unwrap(),
    ));
    let blocks: Vec<_> = (0..NUM_BLOCKS).map(create_block).collect();

    let start = Instant::now();
    let mut tasks = vec![];
    for block in blocks.clone() {
        let store = store.clone();
        tasks.push(tokio::spawn(async move {
            store.lock().await.put(&block).unwrap();
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    let mut tasks = vec![];
    for block in blocks {
        let store = store.clone();
        let cid = block.cid.clone();
        tasks.push(tokio::spawn(async move {
            assert!(store.lock().await.get(&cid).unwrap().is_some());
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    start.elapsed()
}

#[cfg(feature = "persist-rocksdb")]
async fn run_rocks() -> Duration {
    use icn_dag::rocksdb_store::RocksDagStore;
    use icn_dag::StorageService;
    use std::path::PathBuf;
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("rocks");
    let store = Arc::new(Mutex::new(RocksDagStore::new(path).unwrap()));
    let blocks: Vec<_> = (0..NUM_BLOCKS).map(create_block).collect();

    let start = Instant::now();
    let mut tasks = vec![];
    for block in blocks.clone() {
        let store = store.clone();
        tasks.push(tokio::spawn(async move {
            store.lock().await.put(&block).unwrap();
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    let mut tasks = vec![];
    for block in blocks {
        let store = store.clone();
        let cid = block.cid.clone();
        tasks.push(tokio::spawn(async move {
            assert!(store.lock().await.get(&cid).unwrap().is_some());
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    start.elapsed()
}

#[cfg(feature = "persist-sqlite")]
async fn run_sqlite() -> Duration {
    use icn_dag::sqlite_store::SqliteDagStore;
    use icn_dag::StorageService;
    use std::path::PathBuf;
    let dir = tempdir().unwrap();
    let path: PathBuf = dir.path().join("dag.sqlite");
    let store = Arc::new(Mutex::new(SqliteDagStore::new(path).unwrap()));
    let blocks: Vec<_> = (0..NUM_BLOCKS).map(create_block).collect();

    let start = Instant::now();
    let mut tasks = vec![];
    for block in blocks.clone() {
        let store = store.clone();
        tasks.push(tokio::spawn(async move {
            store.lock().await.put(&block).unwrap();
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    let mut tasks = vec![];
    for block in blocks {
        let store = store.clone();
        let cid = block.cid.clone();
        tasks.push(tokio::spawn(async move {
            assert!(store.lock().await.get(&cid).unwrap().is_some());
        }));
    }
    for t in tasks {
        t.await.unwrap();
    }
    start.elapsed()
}

#[cfg(feature = "persist-sled")]
fn bench_sled(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("sled_concurrent", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += run_sled().await;
                }
                total
            })
        });
    });
}

#[cfg(feature = "persist-rocksdb")]
fn bench_rocksdb(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("rocksdb_concurrent", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += run_rocks().await;
                }
                total
            })
        });
    });
}

#[cfg(feature = "persist-sqlite")]
fn bench_sqlite(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("sqlite_concurrent", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += run_sqlite().await;
                }
                total
            })
        });
    });
}

#[cfg(feature = "persist-sled")]
criterion_group!(sled_group, bench_sled);
#[cfg(feature = "persist-rocksdb")]
criterion_group!(rocks_group, bench_rocksdb);
#[cfg(feature = "persist-sqlite")]
criterion_group!(sqlite_group, bench_sqlite);

criterion_main!(
    #[cfg(feature = "persist-sled")]
    sled_group,
    #[cfg(feature = "persist-rocksdb")]
    rocks_group,
    #[cfg(feature = "persist-sqlite")]
    sqlite_group,
);
