import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./notes.css";
import { apiFetch } from "../api/client";
import { useGlobalSearch } from "../search/SearchContext";
export default function Notes() {
    const { normalizedSearchQuery } = useGlobalSearch();
    const [notes, setNotes] = useState([]);
    const [selectedId, setSelectedId] = useState(null);
    const [title, setTitle] = useState("");
    const [content, setContent] = useState("");
    const [saving, setSaving] = useState(false);
    const [status, setStatus] = useState(null);
    // Narrow mode (split pane / small viewport): stack list + editor.
    const mainRef = useRef(null);
    const [isNarrow, setIsNarrow] = useState(false);
    useEffect(() => {
        const el = mainRef.current;
        if (!el)
            return;
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
            const data = await res.json();
            setNotes(data);
        }
        catch (err) {
            console.error(err);
        }
    };
    useEffect(() => {
        void fetchNotes();
    }, []);
    // Drop transient status banners after a moment.
    useEffect(() => {
        if (!status)
            return;
        const t = setTimeout(() => setStatus(null), 1500);
        return () => clearTimeout(t);
    }, [status]);
    // ================= SELECT =================
    const openNew = () => {
        setSelectedId("new");
        setTitle("");
        setContent("");
    };
    const openNote = (note) => {
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
            const saved = await res.json();
            setSelectedId(saved.id);
            setStatus(isNew ? "Created ✓" : "Saved ✓");
            await fetchNotes();
        }
        catch (err) {
            setStatus("Save failed");
        }
        finally {
            setSaving(false);
        }
    };
    // ================= DELETE =================
    const remove = async () => {
        if (selectedId === null || selectedId === "new") {
            closeEditor();
            return;
        }
        if (!confirm("Delete this note?"))
            return;
        try {
            await apiFetch(`/api/notes/${selectedId}`, {
                method: "DELETE",
            });
            closeEditor();
            setStatus("Deleted");
            await fetchNotes();
        }
        catch (err) {
            setStatus("Delete failed");
        }
    };
    const editorOpen = selectedId !== null;
    const showList = !isNarrow || !editorOpen;
    const showEditor = !isNarrow || editorOpen;
    const visibleNotes = normalizedSearchQuery
        ? notes.filter((note) => [note.title ?? "", note.content ?? "", note.updated_at ?? ""]
            .join(" ")
            .toLowerCase()
            .includes(normalizedSearchQuery))
        : notes;
    // ================= UI =================
    return (_jsxs("div", { ref: mainRef, className: `notes ${isNarrow ? "narrow" : ""}`, children: [showList && (_jsxs("div", { className: "notes-list", children: [_jsx("button", { className: "notes-new-btn", onClick: openNew, children: "+ New Note" }), visibleNotes.length === 0 && (_jsx("div", { className: "notes-empty", children: normalizedSearchQuery ? "No notes match your search" : "No notes yet" })), visibleNotes.map((n) => (_jsxs("div", { className: `notes-item ${selectedId === n.id ? "active" : ""}`, onClick: () => openNote(n), children: [_jsx("div", { className: "notes-item-title", children: n.title?.trim() || "Untitled" }), _jsx("div", { className: "notes-item-preview", children: (n.content ?? "").slice(0, 80) })] }, n.id)))] })), showEditor && (_jsx("div", { className: "notes-editor", children: !editorOpen ? (_jsxs("div", { className: "notes-editor-empty", children: [_jsx("div", { className: "notes-editor-empty-icon", children: "\uD83D\uDCDD" }), _jsx("div", { children: "Select a note or create a new one" })] })) : (_jsxs(_Fragment, { children: [_jsxs("div", { className: "notes-editor-header", children: [isNarrow && (_jsx("button", { className: "notes-back-btn", onClick: closeEditor, title: "Back to list", "aria-label": "Back to list", children: "\u2190" })), _jsx("input", { className: "notes-title-input", placeholder: "Title", value: title, onChange: (e) => setTitle(e.target.value) }), status && _jsx("span", { className: "notes-status", children: status })] }), _jsx("textarea", { className: "notes-body-input", placeholder: "Start writing\u2026", value: content, onChange: (e) => setContent(e.target.value) }), _jsxs("div", { className: "notes-editor-actions", children: [_jsx("button", { className: "notes-save-btn", onClick: save, disabled: saving, children: saving ? "Saving…" : "Save" }), selectedId !== "new" && (_jsx("button", { className: "notes-delete-btn", onClick: remove, children: "Delete" })), _jsx("button", { className: "notes-cancel-btn", onClick: closeEditor, children: "Close" })] })] })) }))] }));
}
