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

fn bench_movetex_with_write(c: &mut Criterion) {
    let movetex = Arc::new(Movetex::new(generate_large_string()));

    c.bench_function("Movetex read/write", |b| {
        b.iter(|| {
            let m = Arc::clone(&movetex);
            thread::scope(|s| {
                // Запуск потоков на чтение
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        let _ = m.read();
                    });
                }
                // Одновременная запись длинной строки
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

    c.bench_function("Mutex read/write", |b| {
        b.iter(|| {
            let m = Arc::clone(&mutex);
            thread::scope(|s| {
                // Запуск потоков на чтение
                for _ in 0..10 {
                    let m = Arc::clone(&m);
                    s.spawn(move || {
                        drop(m.lock().unwrap());
                    });
                }
                // Одновременная запись длинной строки
                let m = Arc::clone(&m);
                s.spawn(move || {
                    let mut data = m.lock().unwrap();
                    *data = generate_large_string();
                });
            });
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(6));
    targets = bench_mutex_with_write, bench_movetex_with_write
}

criterion_main!(benches);
