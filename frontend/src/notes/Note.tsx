import { useEffect, useState } from "react";

import "./notes.css";

import { apiFetch } from "@/api/client";

type Note = {
  id: number;
  title: string;
  content: string;
};

export default function Notes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selected, setSelected] = useState<Note | null>(null);

  // ================= FETCH NOTES =================

  useEffect(() => {
    const loadNotes = async () => {
      try {
        const res = await apiFetch("/api/notes");
        const data: Note[] = await res.json();
        setNotes(data);

      } catch (err) {
        console.error(err);
      }
    };
    void loadNotes();
  }, []);

  // ================= CREATE =================

  const createNote = async () => {
    try {
      const res = await apiFetch("/api/notes",
        {
          method: "POST",
          body: JSON.stringify({
            title: "",
            content: "",
          }),
        }
      );
      const newNote: Note = await res.json();
      setNotes((prev) => [
        newNote,
        ...prev,
      ]);
      setSelected(newNote);
    } catch (err) {
      console.error(err);
    }
  };

  // ================= UPDATE (LOCAL STATE) =================

    const handleChange = (value: string) => {if (!selected) {return;}
    const updated = {...selected, content: value};
    setSelected(updated);
    setNotes((prev) =>
      prev.map((n) =>
        n.id === updated.id
          ? updated
          : n
      )
    );
  };

  // ================= AUTOSAVE =================

  useEffect(() => {
    if (!selected) {
      return;
    }

    const timeout = setTimeout(() => {void saveNote(selected);}, 800);
    return () =>
      clearTimeout(timeout);
  }, [selected]);

  const saveNote = async (
    note: Note
  ) => {
    try {
      await apiFetch(`/api/notes/${note.id}`,
        {
          method: "PUT",
          body: JSON.stringify(
            note
          ),
        }
      );

    } catch (err) {
      console.error(err);
    }
  };

  // ================= DELETE =================

  const deleteNote = async (
    id: number
  ) => {
    try {
      await apiFetch(
        `/api/notes/${id}`,
        {
          method: "DELETE",
        }
      );

      setNotes((prev) =>
        prev.filter(
          (n) => n.id !== id
        )
      );

      if (selected?.id === id) {
        setSelected(null);
      }

    } catch (err) {
      console.error(err);
    }
  };

  return (
    <div className="notes-container">

      {/* Sidebar */}

      <div className="notes-sidebar">
        <button onClick={() => void createNote()}>
          + New
        </button>

        {notes.map((n) => (
          <div
            key={n.id}

            onClick={() =>
              setSelected(n)
            }
          >
            {n.title ||
              "Untitled"}
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
                setSelected({
                  ...selected,
                  title:
                    e.target.value,
                })
              }
            />

            <textarea
              value={selected.content}

              onChange={(e) =>
                handleChange(
                  e.target.value
                )
              }
            />

            <button onClick={() => void deleteNote(selected.id)}>
              Delete
            </button>
          </>

        ) : (
          <div>
            Select a note
          </div>
        )}
      </div>
    </div>
  );
}