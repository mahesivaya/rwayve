import { useEffect, useRef, useState } from "react";
import "./notes.css";

import { apiFetch } from "../api/client";

type Note = {
  id: number;
  title: string | null;
  content: string | null;
  updated_at?: string | null;
};


export default function Notes() {
  const [notes, setNotes] = useState<Note[]>([]);
  const [selectedId, setSelectedId] = useState<number | "new" | null>(null);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  // Narrow mode (split pane / small viewport): stack list + editor.
  const mainRef = useRef<HTMLDivElement>(null);
  const [isNarrow, setIsNarrow] = useState(false);
  useEffect(() => {
    const el = mainRef.current;
    if (!el) return;
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setIsNarrow(entry.contentRect.width < 700);
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // ================= LOAD =================
  const fetchNotes = async () => {
    try {
    const res = await apiFetch(`/api/notes`);

    const data: Note[] = await res.json();
    setNotes(data);
    } catch(err)
    {
      console.error(err);
    }
  };

  useEffect(() => {
    void fetchNotes();
  }, []);

  // Drop transient status banners after a moment.
  useEffect(() => {
    if (!status) return;
    const t = setTimeout(() => setStatus(null), 1500);
    return () => clearTimeout(t);
  }, [status]);

  // ================= SELECT =================
  const openNew = () => {
    setSelectedId("new");
    setTitle("");
    setContent("");
  };

  const openNote = (note: Note) => {
    setSelectedId(note.id);
    setTitle(note.title ?? "");
    setContent(note.content ?? "");
  };

  const closeEditor = () => {
    setSelectedId(null);
    setTitle("");
    setContent("");
  };

  // ================= SAVE =================
  const save = async () => {
    if (!title.trim() && !content.trim()) {
      setStatus("Note is empty");
      return;
    }

    setSaving(true);
    try {
      const isNew = selectedId === "new" || selectedId === null;
      const url = isNew
        ? `/api/notes`
        : `/api/notes/${selectedId}`;
      const res = await apiFetch(url, {
        method: isNew ? "POST" : "PUT",
        body: JSON.stringify({ title, content }),
      });

      const saved: Note = await res.json();
      setSelectedId(saved.id);
      setStatus(isNew ? "Created ✓" : "Saved ✓");
      await fetchNotes();
    } catch (err) {
      setStatus("Save failed");
    } finally {
      setSaving(false);
    }
  };

  // ================= DELETE =================
  const remove = async () => {
    if (selectedId === null || selectedId === "new") {
      closeEditor();
      return;
    }
    if (!confirm("Delete this note?")) return;
    try {
      await apiFetch(`/api/notes/${selectedId}`, {
        method: "DELETE",
      });

    closeEditor();
    setStatus("Deleted");
    await fetchNotes();
    } catch(err)
    {
      setStatus("Delete failed")
    }
  };

  const editorOpen = selectedId !== null;
  const showList = !isNarrow || !editorOpen;
  const showEditor = !isNarrow || editorOpen;

  // ================= UI =================
  return (
    <div ref={mainRef} className={`notes ${isNarrow ? "narrow" : ""}`}>
      {/* LIST */}
      {showList && (
        <div className="notes-list">
          <button className="notes-new-btn" onClick={openNew}>
            + New Note
          </button>

          {notes.length === 0 && (
            <div className="notes-empty">No notes yet</div>
          )}

          {notes.map((n) => (
            <div
              key={n.id}
              className={`notes-item ${selectedId === n.id ? "active" : ""}`}
              onClick={() => openNote(n)}
            >
              <div className="notes-item-title">
                {n.title?.trim() || "Untitled"}
              </div>
              <div className="notes-item-preview">
                {(n.content ?? "").slice(0, 80)}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* EDITOR */}
      {showEditor && (
        <div className="notes-editor">
          {!editorOpen ? (
            <div className="notes-editor-empty">
              <div className="notes-editor-empty-icon">📝</div>
              <div>Select a note or create a new one</div>
            </div>
          ) : (
            <>
              <div className="notes-editor-header">
                {isNarrow && (
                  <button
                    className="notes-back-btn"
                    onClick={closeEditor}
                    title="Back to list"
                    aria-label="Back to list"
                  >
                    ←
                  </button>
                )}
                <input
                  className="notes-title-input"
                  placeholder="Title"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                />
                {status && <span className="notes-status">{status}</span>}
              </div>

              <textarea
                className="notes-body-input"
                placeholder="Start writing…"
                value={content}
                onChange={(e) => setContent(e.target.value)}
              />

              <div className="notes-editor-actions">
                <button
                  className="notes-save-btn"
                  onClick={save}
                  disabled={saving}
                >
                  {saving ? "Saving…" : "Save"}
                </button>
                {selectedId !== "new" && (
                  <button className="notes-delete-btn" onClick={remove}>
                    Delete
                  </button>
                )}
                <button className="notes-cancel-btn" onClick={closeEditor}>
                  Close
                </button>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}
