use std::sync::{Arc, Mutex};
use warp::Filter;

#[tokio::main]
// головна асинхронна функція
async fn main() {
    
    let last_result = Arc::new(Mutex::new(None));

    // маршрут для обчислення виразу
    let calc_route = warp::path("calculate")
        .and(warp::post())
        .and(warp::body::form())
        .and(with_last_result(last_result.clone()))
        .and_then(handle_calculate);

    // маршрут для інтерфейсу
    let ui_route = warp::path::end()
        .map(|| warp::reply::html(INTERFACE_HTML));

    // запуск сервера на локальному хості
    let routes = ui_route.or(calc_route);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with_last_result(
    last_result: Arc<Mutex<Option<i32>>>,
) -> impl Filter<Extract = (Arc<Mutex<Option<i32>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || last_result.clone())
}

// функція для обробки запиту
async fn handle_calculate(
    form: std::collections::HashMap<String, String>,
    last_result: Arc<Mutex<Option<i32>>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let input = form.get("expression").unwrap_or(&"".to_string()).clone();
    let mut last_result_lock = last_result.lock().unwrap();
    let result = match calculate(&input, *last_result_lock) {
        Ok(res) => {
            *last_result_lock = Some(res);
            format!("Результат: {}. Ведіть last + ваше значення для використання цього результату", res)
        }
        Err(e) => format!("Помилка: {}", e),
    };
    Ok(warp::reply::html(result))
}

// функція для обчислення виразу
fn calculate(input: &str, last_result: Option<i32>) -> Result<i32, String> {
    let mut tokens = input.split_whitespace().peekable(); // Розбиваємо вираз на частини
    let mut result: i32;

    // Перевіряємо перший елемент (може бути числом або "last")
    match tokens.next() {
        Some("last") => {
            result = last_result.ok_or("Немає збереженого результату.".to_string())?;
        }
        Some(num_str) => {
            result = num_str.parse::<i32>().map_err(|_| "Некоректний перший аргумент.".to_string())?;
        }
        None => return Err("Ви не ввели вираз.".to_string()),
    }

    // Обробляємо залишок виразу
    while let Some(op) = tokens.next() {
        // Отримуємо наступне число
        let num_str = tokens.next().ok_or("Немає другого аргументу після оператора.".to_string())?;
        let num = num_str.parse::<i32>().map_err(|_| "Некоректний аргумент.".to_string())?;

        // Виконуємо операцію залежно від оператора
        result = match op {
            "+" => result + num,
            "-" => result - num,
            "*" => result * num,
            "/" => {
                if num == 0 {
                    return Err("Помилка: ділення на нуль!".to_string());
                }
                result / num
            }
            _ => return Err(format!("Невідомий оператор: {}", op)),
        };
    }

    Ok(result)
}



// HTML інтерфейс
static INTERFACE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="uk">
<head>
    <meta charset="UTF-8">
    <title>Калькулятор</title>
</head>
<body>
    <h1>Лабораторна робота №2. Калькулятор</h1>
    <form id="calc-form">
        <label for="expression">Введіть вираз:</label>
        <input type="text" id="expression" name="expression" required>
        <button type="button" onclick="calculate()">Обчислити</button>
    </form>
    <p id="result"></p>

    <script>
        async function calculate() {
            const expression = document.getElementById("expression").value;
            const response = await fetch("/calculate", {
                method: "POST",
                headers: {
                    "Content-Type": "application/x-www-form-urlencoded",
                },
                body: new URLSearchParams({ expression })
            });
            const result = await response.text();
            document.getElementById("result").innerText = result;
        }
    </script>
</body>
</html>
"#;
