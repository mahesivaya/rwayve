import { FormEvent, useMemo, useState } from "react";
import { useGlobalSearch } from "../search/SearchContext";
import "./tasks.css";

type Task = {
  id: number;
  name: string;
  description: string;
  createdAt: string;
};

export default function Tasks() {
  const { normalizedSearchQuery } = useGlobalSearch();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [creating, setCreating] = useState(false);
  const [taskName, setTaskName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState("");

  const visibleTasks = useMemo(() => {
    if (!normalizedSearchQuery) return tasks;

    return tasks.filter((task) =>
      [task.name, task.description]
        .join(" ")
        .toLowerCase()
        .includes(normalizedSearchQuery)
    );
  }, [normalizedSearchQuery, tasks]);

  const createTask = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const name = taskName.trim();
    const details = description.trim();

    if (!name) {
      setError("Task name is required");
      return;
    }

    setTasks((prev) => [
      {
        id: Date.now(),
        name,
        description: details,
        createdAt: new Date().toISOString(),
      },
      ...prev,
    ]);
    setTaskName("");
    setDescription("");
    setError("");
    setCreating(false);
  };

  return (
    <div className="tasks-app">
      <aside className="tasks-sidebar">
        <button
          className="create-task-btn"
          onClick={() => {
            setCreating(true);
            setError("");
          }}
        >
          + Create task
        </button>

        <div className="task-filter-title">Tasks</div>
        <button className="task-filter active">All tasks</button>
        <button className="task-filter">Created by me</button>
        <button className="task-filter">Recently added</button>
      </aside>

      <main className="tasks-main">
        <div className="tasks-header">
          <div>
            <h2>Tasks</h2>
            <p>Create simple work items with a name and description.</p>
          </div>
          <span className="tasks-count">{tasks.length} total</span>
        </div>

        {creating && (
          <form className="task-create-form" onSubmit={createTask}>
            <div className="task-form-grid">
              <label>
                <span>Task name</span>
                <input
                  value={taskName}
                  onChange={(event) => setTaskName(event.target.value)}
                  placeholder="Enter task name"
                  autoFocus
                />
              </label>

              <label>
                <span>Description</span>
                <textarea
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                  placeholder="Add task details"
                />
              </label>
            </div>

            {error && <div className="task-error">{error}</div>}

            <div className="task-form-actions">
              <button
                type="button"
                onClick={() => {
                  setCreating(false);
                  setTaskName("");
                  setDescription("");
                  setError("");
                }}
              >
                Cancel
              </button>
              <button type="submit" className="primary">
                Create task
              </button>
            </div>
          </form>
        )}

        <div className="task-list">
          {visibleTasks.length === 0 ? (
            <div className="tasks-empty">
              <strong>{tasks.length === 0 ? "No tasks yet" : "No matching tasks"}</strong>
              <span>
                {tasks.length === 0
                  ? "Use + Create task to add your first task."
                  : "Try a different search term."}
              </span>
            </div>
          ) : (
            visibleTasks.map((task) => (
              <article key={task.id} className="task-card">
                <div>
                  <h3>{task.name}</h3>
                  <p>{task.description || "No description added."}</p>
                </div>
                <time dateTime={task.createdAt}>
                  {new Date(task.createdAt).toLocaleDateString()}
                </time>
              </article>
            ))
          )}
        </div>
      </main>
    </div>
  );
}
