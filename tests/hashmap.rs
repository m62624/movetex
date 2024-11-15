use movetex::Movetex;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

fn generate_random_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

#[test]
fn test_movetex_parallel_write_consistency() {
    // Эталонный результат с последовательной загрузкой
    let mut reference_map = HashMap::new();
    let mut prefilled_values = Vec::new();

    for i in 0..10 {
        for j in 0..1000 {
            let key = i * 1000 + j;
            let value = generate_random_string();
            prefilled_values.push((key, value.clone()));
            reference_map.insert(key, value);
        }
    }

    // Movetex для параллельной загрузки
    let movetex_map = Arc::new(Movetex::new(HashMap::new()));

    let movetex_map_writer = Arc::clone(&movetex_map);
    let movetex_map_reader = Arc::clone(&movetex_map);

    // Barrier to synchronize threads
    let barrier = Arc::new(Barrier::new(11)); // 10 writer threads + 1 main thread

    // Создаем 10 потоков, каждый записывает по 1000 элементов
    thread::scope(|s| {
        for i in 0..10 {
            let m = Arc::clone(&movetex_map_writer);
            let values = prefilled_values.clone();
            let c = Arc::clone(&barrier);

            s.spawn(move || {
                for (key, value) in values.iter().skip(i * 1000).take(1000) {
                    loop {
                        if m.write(|map| {
                            map.insert(*key, value.clone());
                        }) {
                            break;
                        }
                        thread::yield_now();
                    }
                }
                c.wait(); // Wait at the barrier
            });
        }

        barrier.wait(); // Main thread waits for all writer threads
        println!("All writes to movetex are completed.");
    });

    // Чтение через 5 секунд и проверка идентичности
    let start_time = Instant::now();
    let time_limit = Duration::from_secs(60);

    let reader_thread = thread::spawn(move || {
        thread::sleep(Duration::from_secs(5)); // Задержка перед началом чтения
        loop {
            let elapsed = start_time.elapsed();
            if elapsed > time_limit {
                panic!(
                    "Test failed: Took longer than {} seconds to complete",
                    time_limit.as_secs()
                );
            }

            if *movetex_map_reader.read() == reference_map {
                println!("Test passed: HashMaps are identical.");
                return;
            }
        }
    });

    reader_thread.join().expect("Reader thread panicked");
}
