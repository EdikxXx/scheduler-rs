use scheduler_rs::tasks::containers::HandlerRegistry;
use scheduler_rs::tasks::types::{BoxedFuture, Job, Task, TaskKind};
use std::time::Duration;
use tokio::main;

#[main]
async fn main() {
    let mut registry = HandlerRegistry::new();

    // СЛУЧАЙ 1: Синхронная функция с аргументами (add_task)
    // Мы должны "захватить" аргументы 10 и 20 внутрь.
    // И мы должны обработать результат i64, так как Job ожидает возврата ()
    let job_sync = Box::new(|| -> BoxedFuture {
        // Оборачиваем синхронный код в Box::pin(async { ... })
        Box::pin(async {
            let res = add_task(10, 20);
            println!("Result wrapper: {}", res);
        })
    });

    registry.insert_task("math_job", job_sync);

    // Важно: String должна принадлежать замыканию, поэтому .to_string()
    // СЛУЧАЙ 2: Асинхронная функция с аргументами (show_file)
    let path = "config.toml".to_string();

    let job_async = Box::new(move || -> BoxedFuture {
        let path_clone = path.clone(); // Клонируем для использования внутри async
        Box::pin(async move {
            show_file(&path_clone).await;
        })
    });

    registry.insert_task("io_job", job_async);

    // Проверка
    println!("--- Running math ---");
    registry.run("math_job").await;

    println!("--- Running io ---");
    registry.run("io_job").await;

    println!("Registry state: {:#?}", registry.map.keys());

    let mut task = Task::new("test_task", TaskKind::Interval(Duration::from_hours(2)));
}

fn add_task(a: i64, b: i64) -> i64 {
    println!("Calculating sync...");
    a + b
}

async fn show_file(p: &str) {
    // Имитация чтения файла
    println!("Reading file: {}", p);
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!("File contents loaded");
}
