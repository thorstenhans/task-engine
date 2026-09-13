const list = document.getElementById("tasks");
const form = document.getElementById("new-task");
const titleInput = document.getElementById("title");

async function loadTasks() {
  const res = await fetch("/api/tasks");
  const tasks = await res.json();
  list.innerHTML = "";
  for (const task of tasks) {
    const li = document.createElement("li");
    li.className = task.status === "done" ? "done" : "";
    li.textContent = task.title;

    const btn = document.createElement("button");
    btn.textContent = task.status === "done" ? "Reopen" : "Done";
    btn.onclick = () => toggle(task);
    li.appendChild(btn);
    list.appendChild(li);
  }
}

async function toggle(task) {
  const status = task.status === "done" ? "open" : "done";
  await fetch(`/api/tasks/${task.id}`, {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ status }),
  });
  loadTasks();
}

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  await fetch("/api/tasks", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ title: titleInput.value }),
  });
  titleInput.value = "";
  loadTasks();
});

loadTasks();
