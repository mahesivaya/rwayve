import { useEffect, useState } from "react";

import "./notes.css";

import {
  createNoteApi,
  deleteNoteApi,
  getNotes,
  updateNoteApi,
  type Note,
} from "../api/notes";

type EditableNote = Note & {
  title: string;
  content: string;
};

export default function Notes() {
  const [notes, setNotes] = useState<EditableNote[]>([]);
  const [selected, setSelected] = useState<EditableNote | null>(null);

  // ================= FETCH NOTES =================

  useEffect(() => {
    const loadNotes = async () => {
      try {
        const data = await getNotes();
        setNotes(
          data.map((note) => ({
            ...note,
            title: note.title ?? "",
            content: note.content ?? "",
          }))
        );

      } catch (err) {
        console.error(err);
      }
    };
    void loadNotes();
  }, []);

  // ================= CREATE =================

  const createNote = async () => {
    try {
      const saved = await createNoteApi({
        title: "",
        content: "",
      });
      const newNote: EditableNote = {
        ...saved,
        title: saved.title ?? "",
        content: saved.content ?? "",
      };
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
    note: EditableNote
  ) => {
    try {
      await updateNoteApi(note.id, {
        title: note.title,
        content: note.content,
      });

    } catch (err) {
      console.error(err);
    }
  };

  // ================= DELETE =================

  const deleteNote = async (
    id: number
  ) => {
    try {
      await deleteNoteApi(id);

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
