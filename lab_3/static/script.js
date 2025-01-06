async function fetchTasks() {
    const response = await fetch('/tasks');
    const tasks = await response.json();
    const container = document.getElementById('tasks-container');
    container.innerHTML = tasks.map(task => `
        <div>
            <input type="checkbox" ${task.status ? 'checked' : ''} onchange="toggleTask(${task.id})">
            <span>${task.description}</span>
            <button onclick="deleteTask(${task.id})">Видалити</button>
        </div>
    `).join('');
}

async function addTask() {
    const desc = document.getElementById('task-desc').value;
    if (!desc) {
        alert('Введіть опис завдання');
        return;
    }

    // Відправляємо POST-запит на сервер
    const response = await fetch('/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            description: desc,  // Опис завдання
            status: false       // Початковий статус
        })
    });

    if (response.ok) {
        alert('Завдання додано!');
        document.getElementById('task-desc').value = ''; // Очищення поля вводу
        fetchTasks(); // Оновлення списку завдань
    } else {
        alert('Помилка при додаванні завдання!');
    }
}


async function toggleTask(id) {
    await fetch(`/tasks/${id}/toggle`, { method: 'PUT' });
    fetchTasks();
}

async function deleteTask(id) {
    await fetch(`/tasks/${id}`, { method: 'DELETE' });
    fetchTasks();
}



document.getElementById('add-task').addEventListener('click', addTask);
fetchTasks();

async function uploadFile() {
    const fileInput = document.getElementById('task-file');
    const file = fileInput.files[0];

    if (!file) {
        alert('Виберіть файл для завантаження!');
        return;
    }

    console.log('Файл для завантаження:', file);

    const formData = new FormData();
    formData.append('file', file); // Ім'я "file" має відповідати очікуваному на сервері

    try {
        const response = await fetch('/upload', {
            method: 'POST',
            body: formData,
        });

        if (response.ok) {
            console.log('Файл завантажено успішно');
            alert('Файл успішно завантажено!');
            fetchTasks(); // Оновлення списку завдань
        } else {
            console.error('Помилка при завантаженні файлу:', response.statusText);
            alert('Помилка при завантаженні файлу!');
        }
    } catch (error) {
        console.error('Помилка при виконанні запиту:', error);
        alert('Помилка при виконанні запиту');
    }
}

document.getElementById('upload-file').addEventListener('click', uploadFile);
