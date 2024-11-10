use std::sync::atomic::AtomicPtr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

pub struct Movetex<T> {
    data: AtomicPtr<T>,            // атомарный указатель на данные
    write_in_progress: AtomicBool, // флаг для записи
}

impl<T> Movetex<T> {
    // Создаем новую обертку Arc<Movetex> с начальным значением
    pub fn new(value: T) -> Arc<Self> {
        Arc::new(Self {
            data: AtomicPtr::new(Box::into_raw(Box::new(value))),
            write_in_progress: AtomicBool::new(false),
        })
    }

    // Получаем слабую ссылку на Movetex
    pub fn share(this: &Arc<Self>) -> Weak<Self> {
        Arc::downgrade(this)
    }

    // Чтение: пока запись не в процессе, возвращаем ссылку на данные
    pub fn read(&self) -> Option<&T> {
        if !self.write_in_progress.load(Ordering::Acquire) {
            // Безопасно, так как нет активной записи
            unsafe { self.data.load(Ordering::Acquire).as_ref() }
        } else {
            None // Если идет запись, вернем None
        }
    }

    // Запись: забираем значение для модификации, затем возвращаем
    pub fn write(&self, new_value: T) -> Result<(), T> {
        // Пробуем установить флаг для записи
        if self
            .write_in_progress
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            // Захватываем данные для записи (move)
            let old_ptr = self.data.load(Ordering::Acquire);
            if !old_ptr.is_null() {
                // Забираем старые данные
                unsafe {
                    drop(Box::from_raw(old_ptr)); // Освобождаем старые данные
                }
            }

            // Создаем новое значение и обновляем указатель
            self.data
                .store(Box::into_raw(Box::new(new_value)), Ordering::Release);

            // Сбрасываем флаг записи
            self.write_in_progress.store(false, Ordering::Release);

            Ok(())
        } else {
            Err(new_value) // Если не удалось начать запись, возвращаем данные
        }
    }
}

#[test]
// Пример использования
fn test_t_0() {
    let data = Movetex::new(42);

    // Чтение значения
    if let Some(value) = data.read() {
        println!("Read value: {}", value);
    }

    // Запись нового значения
    if data.write(100).is_ok() {
        println!("Value updated.");
    }

    // Чтение после обновления
    if let Some(value) = data.read() {
        println!("Read value after update: {}", value);
    }
}
