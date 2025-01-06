mod tasks;
mod file_handler;

use warp::Filter;
use tasks::{Task, toggle_task_status, delete_task};
use file_handler::{load_tasks, save_tasks};
use std::sync::{Arc, Mutex};
use futures::{StreamExt, TryStreamExt};
use bytes::Buf;
use warp::reject;

type Tasks = Arc<Mutex<Vec<Task>>>;

#[tokio::main]
async fn main() {
    let file_path = "tasks.txt";
    let tasks = Arc::new(Mutex::new(load_tasks(file_path)));

    let list_tasks = warp::path("tasks")
        .and(warp::get())
        .and(with_tasks(tasks.clone()))
        .map(|tasks: Tasks| {
            let tasks = tasks.lock().unwrap();
            warp::reply::json(&*tasks)
        });

    let add_task_route = warp::path("tasks")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_tasks(tasks.clone()))
        .map(|mut new_task: Task, tasks: Tasks| {
            let mut tasks = tasks.lock().unwrap();
            new_task.id = Some(
                tasks.last().map_or(1, |last_task| last_task.id.unwrap_or(0) + 1)
            );
            tasks.push(new_task.clone());
            warp::reply::json(&*tasks)
        });

    let toggle_task_route = warp::path!("tasks" / u32 / "toggle")
        .and(warp::put())
        .and(with_tasks(tasks.clone()))
        .map(|id: u32, tasks: Tasks| {
            let mut tasks = tasks.lock().unwrap();
            toggle_task_status(&mut *tasks, id);
            warp::reply::json(&*tasks)
        });

    let delete_task_route = warp::path!("tasks" / u32)
        .and(warp::delete())
        .and(with_tasks(tasks.clone()))
        .map(|id: u32, tasks: Tasks| {
            let mut tasks = tasks.lock().unwrap();
            delete_task(&mut *tasks, id);
            warp::reply::json(&*tasks)
        });

    let upload_file_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(20 * 1024 * 1024)) // Максимальний розмір файлу: 20 МБ
        .and(with_tasks(tasks.clone()))
        .and_then(|form: warp::multipart::FormData, tasks: Tasks| async move {
            println!("Отримано запит на /upload"); 
            let result = form
                .map(|part_result| async move {
                    let part = part_result.map_err(|_| {
                        println!("Помилка при отриманні частини форми"); 
                        warp::reject()
                    })?;
    
                    println!("Обробка частини форми: {:?}", part.name()); 
    
                    let mut buf = Vec::new();
                    let mut stream = part.stream();
    
                    while let Some(chunk) = stream.try_next().await.map_err(|_| {
                        println!("Помилка при читанні потоку"); 
                        warp::reject()
                    })? {
                        buf.extend_from_slice(chunk.chunk());
                    }
    
                    println!("Розмір зчитаного буфера: {}", buf.len());
    
                    Ok::<Vec<u8>, warp::Rejection>(buf)
                })
                .buffer_unordered(10)
                .try_collect::<Vec<_>>()
                .await;
    
            match result {
                Ok(parts) => {
                    println!("Успішно отримано {} частин", parts.len()); 
    
                    for buf in parts {
                        let content = String::from_utf8(buf).unwrap_or_default();
                        println!("Вміст файлу: {}", content); 
    
                        for line in content.lines() {
                            let parts: Vec<&str> = line.split(';').collect();
                            if parts.len() == 3 {
                                let mut task_list = tasks.lock().unwrap();
    
                                // Отримуємо довжину перед тим, як додавати нове завдання
                                let new_id = task_list.len() as u32 + 1;
    
                                println!(
                                    "Додаємо завдання: id={}, description={}, status={}",
                                    new_id, parts[1], parts[2]
                                ); // Діагностика: нове завдання
    
                                task_list.push(Task {
                                    id: Some(new_id),
                                    description: parts[1].to_string(),
                                    status: parts[2] == "true",
                                });
                            } else {
                                println!("Пропущено рядок через невірний формат: {}", line); 
                            }
                        }
                    }
    
                    println!("Список завдань оновлено"); 
                    Ok::<_, warp::Rejection>(warp::reply::json(&*tasks.lock().unwrap()))
                }
                Err(_) => {
                    println!("Помилка під час обробки завантаження"); 
                    Err(warp::reject::custom(UploadError))
                }
            }
        });

    
    

    let static_files = warp::fs::dir("static");

    let routes = list_tasks
        .or(add_task_route)
        .or(toggle_task_route)
        .or(delete_task_route)
        .or(upload_file_route)
        .or(warp::get().and(static_files));


    println!("Сервер запущено на http://127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    save_tasks(file_path, &*tasks.lock().unwrap());
}

fn with_tasks(tasks: Tasks) -> impl Filter<Extract = (Tasks,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tasks.clone())
}

#[derive(Debug)]
struct UploadError;
impl warp::reject::Reject for UploadError {}
