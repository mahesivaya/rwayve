import { useEffect, useState } from "react";

type Note = {
  id: number;
  title: string;
  content: string;
};

import {API_BASE} from "../utils/env";

export default function Notes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<Note | null>(null);

  // ================= FETCH NOTES =================
  useEffect(() => {
    fetch(`${API_BASE}/api/notes`, {
      headers: {
        Authorization: `Bearer ${localStorage.getItem("token")}`,
      },
    })
      .then((res) => res.json())
      .then(setNotes);
  }, []);

  // ================= CREATE =================
  const createNote = async () => {
    const res = await fetch(`${API_BASE}/api/notes`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${localStorage.getItem("token")}`,
      },
      body: JSON.stringify({ title: "", content: "" }),
    });

    const newNote = await res.json();
    setNotes((prev) => [newNote, ...prev]);
    setSelected(newNote);
  };

  // ================= UPDATE (LOCAL STATE) =================
  const handleChange = (value: string) => {
    if (!selected) return;

    const updated = { ...selected, content: value };

    setSelected(updated);

    setNotes((prev) =>
      prev.map((n) => (n.id === updated.id ? updated : n))
    );
  };

  // ================= AUTOSAVE =================
  useEffect(() => {
    if (!selected) return;

    const timeout = setTimeout(() => {
      saveNote(selected);
    }, 800);

    return () => clearTimeout(timeout);
  }, [selected?.content]);

  const saveNote = async (note: Note) => {
    await fetch(`${API_BASE}/api/notes/${note.id}`, {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${localStorage.getItem("token")}`,
      },
      body: JSON.stringify(note),
    });
  };

  // ================= DELETE =================
  const deleteNote = async (id: number) => {
    await fetch(`${API_BASE}/api/notes/${id}`, {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${localStorage.getItem("token")}`,
      },
    });

    setNotes((prev) => prev.filter((n) => n.id !== id));
    if (selected?.id === id) setSelected(null);
  };

  return (
    <div className="notes-container">
      {/* Sidebar */}
      <div className="notes-sidebar">
        <button onClick={createNote}>+ New</button>

        {notes.map((n) => (
          <div key={n.id} onClick={() => setSelected(n)}>
            {n.title || "Untitled"}
          </div>
        ))}
      </div>

      {/* Editor */}
      <div className="notes-editor">
        {selected ? (
          <>
            <input
              placeholder="Title"
              value={selected.title}
              onChange={(e) =>
                setSelected({ ...selected, title: e.target.value })
              }
            />

            <textarea
              value={selected.content}
              onChange={(e) => handleChange(e.target.value)}
            />

            <button onClick={() => deleteNote(selected.id)}>Delete</button>
          </>
        ) : (
          <div>Select a note</div>
        )}
      </div>
    </div>
  );
}