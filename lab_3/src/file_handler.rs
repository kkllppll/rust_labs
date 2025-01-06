use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::tasks::Task;

pub fn load_tasks(file_path: &str) -> Vec<Task> {
    let file = File::open(file_path).unwrap_or_else(|_| {
        println!("Файл не знайдено, створюємо новий файл: {}", file_path);
        File::create(file_path).expect("Не вдалося створити файл")
    });
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();

    for line in reader.lines() {
        if let Ok(task_line) = line {
            let parts: Vec<&str> = task_line.split(';').collect();
            if parts.len() == 3 {
                tasks.push(Task {
                    id: parts[0].parse().ok(),
                    description: parts[1].to_string(),
                    status: parts[2] == "true",
                });
            } else {
                println!("Пропущено рядок через неправильний формат: {}", task_line);
            }
        }
    }
    tasks
}

pub fn save_tasks(file_path: &str, tasks: &Vec<Task>) {
    let mut file = File::create(file_path).expect("Не вдалося створити файл для запису");
    for task in tasks {
        if let Err(err) = writeln!(
            file,
            "{};{};{}",
            task.id.map_or("".to_string(), |id| id.to_string()),
            task.description,
            task.status
        ) {
            println!("Помилка запису завдання: {:?}", err);
        }
    }
}
