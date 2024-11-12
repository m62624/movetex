use criterion::{criterion_group, criterion_main, Criterion};
use movetex::Movetex;
use rand::{distributions::Alphanumeric, Rng};
use std::sync::{Arc, Mutex};
use std::thread;

fn generate_large_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(1_000_000)
        .map(char::from)
        .collect()
}

fn bench_mutex_with_write_small(c: &mut Criterion) {
    let mutex = Arc::new(Mutex::new(0u128));

    c.bench_function("Mutex read/write small", |b| {
        b.iter(|| {
            let m = Arc::clone(&mutex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        drop(m.lock().unwrap());
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    let mut data = m.lock().unwrap();
                    *data = 1;
                });
            });
        })
    });
}

fn bench_movetex_with_write_small(c: &mut Criterion) {
    let movetex = Arc::new(Movetex::new(0u128));

    c.bench_function("Movetex read/write small", |b| {
        b.iter(|| {
            let m = Arc::clone(&movetex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        let _ = m.read();
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    m.write(|data| {
                        *data = 1;
                    });
                });
            });
        })
    });
}

fn bench_movetex_with_write(c: &mut Criterion) {
    let movetex = Arc::new(Movetex::new(generate_large_string()));

    c.bench_function("Movetex read/write large", |b| {
        b.iter(|| {
            let m = Arc::clone(&movetex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        let _ = m.read();
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    m.write(|data| {
                        *data = generate_large_string();
                    });
                });
            });
        })
    });
}

fn bench_mutex_with_write(c: &mut Criterion) {
    let mutex = Arc::new(Mutex::new(generate_large_string()));

    c.bench_function("Mutex read/write large", |b| {
        b.iter(|| {
            let m = Arc::clone(&mutex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        drop(m.lock().unwrap());
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    let mut data = m.lock().unwrap();
                    *data = generate_large_string();
                });
            });
        })
    });
}

fn bench_movetex_with_multiple_writes(c: &mut Criterion) {
    let movetex = Arc::new(Movetex::new(generate_large_string()));

    c.bench_function("Movetex multiple writes", |b| {
        b.iter(|| {
            let m = Arc::clone(&movetex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        m.write(|data| {
                            *data = generate_large_string();
                        });
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    let _ = m.read();
                });
            });
        })
    });
}

fn bench_mutex_with_multiple_writes(c: &mut Criterion) {
    let mutex = Arc::new(Mutex::new(generate_large_string()));

    c.bench_function("Mutex multiple writes", |b| {
        b.iter(|| {
            let m = Arc::clone(&mutex);
            thread::scope(|s| {
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        let mut data = m.lock().unwrap();
                        *data = generate_large_string();
                    });
                }
                let m = Arc::clone(&m);
                s.spawn(move || {
                    drop(m.lock().unwrap());
                });
            });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(6));
    targets = bench_mutex_with_write_small, bench_movetex_with_write_small,
    bench_mutex_with_write, bench_movetex_with_write, bench_mutex_with_multiple_writes, bench_movetex_with_multiple_writes
}

criterion_main!(benches);
