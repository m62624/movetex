use movetex::Movetex;
use std::sync::Arc;
use std::time;

mod read_tests {
    use super::*;

    #[test]
    fn test_t_0() {
        let movetex = Movetex::new(42);
        assert_eq!(*movetex.read(), 42);
    }

    #[test]
    fn test_t_1() {
        let movetex = Arc::new(Movetex::new(42));

        std::thread::scope(|s| {
            for _ in 0..10 {
                let m1 = movetex.clone();
                let m2 = movetex.clone();

                s.spawn(move || {
                    assert_eq!(*m1.read(), 42);
                });

                s.spawn(move || {
                    assert_eq!(*m2.read(), 42);
                });
            }
        });
    }
}

mod write_tests {
    use super::*;

    #[test]
    fn test_t_0() {
        let movetex = Movetex::new(42);
        assert!(movetex.write(|value| {
            *value = 43;
        }));
        assert_eq!(*movetex.read(), 43);
    }

    #[test]
    fn test_t_1() {
        std::thread::scope(|s| {
            let movetex = Arc::new(Movetex::new(42));

            let m1 = movetex.clone();
            let m2 = movetex.clone();

            s.spawn(move || {
                m1.write(|value| {
                    *value = 43;
                    std::thread::sleep(time::Duration::from_secs(2));
                });
            });
            s.spawn(move || {
                std::thread::sleep(time::Duration::from_secs(1));
                assert_eq!(m2.write(|_| {}), false);
            });
        });
    }

    #[test]
    fn test_t_2() {
        let movetex = Arc::new(Movetex::new(String::from("42")));
        std::thread::scope(|s| {
            let m1 = movetex.clone();
            let m2 = movetex.clone();
            let m3 = movetex.clone();

            s.spawn(move || {
                m1.write(|_| {
                    std::thread::sleep(time::Duration::from_secs(3));
                });
            });

            s.spawn(move || {
                std::thread::sleep(time::Duration::from_secs(1));
                m2.write(|value| {
                    *value = String::from("43");
                });
            });

            s.spawn(move || {
                std::thread::sleep(time::Duration::from_secs(1));
                m3.write(|value| {
                    *value = String::from("44");
                });
            });
        });

        assert_eq!(*movetex.read(), "42");
    }
}

mod swap_tests {
    use super::*;

    #[test]
    fn test_t_0() {
        let movetex = Movetex::new(42);

        assert_eq!(movetex.swap(43), Some(42));
    }

    #[test]
    fn test_t_1() {
        let movetex = Arc::new(Movetex::new(42));

        std::thread::scope(|s| {
            let m1 = movetex.clone();
            let m2 = movetex.clone();

            s.spawn(move || {
                assert_eq!(m1.swap(43), Some(42));
            });

            s.spawn(move || {
                assert_eq!(m2.swap(44), Some(43));
            });
        });
    }
}
