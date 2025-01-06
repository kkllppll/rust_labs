use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<u32>, // `Option` дозволяє, щоб поле могло бути відсутнє
    pub description: String,
    pub status: bool, // true = виконано, false = не виконано
}

pub fn add_task(tasks: &mut Vec<Task>, description: &str) {
    let id = if let Some(last) = tasks.last() {
        last.id.unwrap_or(0) + 1 // Отримуємо останній `id`, якщо він є
    } else {
        1 // Якщо список порожній, починаємо з 1
    };
    tasks.push(Task {
        id: Some(id), // Додаємо нове `id` в `Some`
        description: description.to_string(),
        status: false,
    });
}

pub fn toggle_task_status(tasks: &mut Vec<Task>, id: u32) {
    if let Some(task) = tasks.iter_mut().find(|t| t.id == Some(id)) {
        task.status = !task.status;
    }
}

pub fn delete_task(tasks: &mut Vec<Task>, id: u32) {
    tasks.retain(|task| task.id != Some(id));
}
