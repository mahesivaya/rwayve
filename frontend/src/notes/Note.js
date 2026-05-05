import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useState } from "react";
const API_BASE = import.meta.env.VITE_API_URL;
export default function Notes() {
    const [notes, setNotes] = useState([]);
    const [selected, setSelected] = useState(null);
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
    const handleChange = (value) => {
        if (!selected)
            return;
        const updated = { ...selected, content: value };
        setSelected(updated);
        setNotes((prev) => prev.map((n) => (n.id === updated.id ? updated : n)));
    };
    // ================= AUTOSAVE =================
    useEffect(() => {
        if (!selected)
            return;
        const timeout = setTimeout(() => {
            saveNote(selected);
        }, 800);
        return () => clearTimeout(timeout);
    }, [selected?.content]);
    const saveNote = async (note) => {
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
    const deleteNote = async (id) => {
        await fetch(`${API_BASE}/api/notes/${id}`, {
            method: "DELETE",
            headers: {
                Authorization: `Bearer ${localStorage.getItem("token")}`,
            },
        });
        setNotes((prev) => prev.filter((n) => n.id !== id));
        if (selected?.id === id)
            setSelected(null);
    };
    return (_jsxs("div", { className: "notes-container", children: [_jsxs("div", { className: "notes-sidebar", children: [_jsx("button", { onClick: createNote, children: "+ New" }), notes.map((n) => (_jsx("div", { onClick: () => setSelected(n), children: n.title || "Untitled" }, n.id)))] }), _jsx("div", { className: "notes-editor", children: selected ? (_jsxs(_Fragment, { children: [_jsx("input", { placeholder: "Title", value: selected.title, onChange: (e) => setSelected({ ...selected, title: e.target.value }) }), _jsx("textarea", { value: selected.content, onChange: (e) => handleChange(e.target.value) }), _jsx("button", { onClick: () => deleteNote(selected.id), children: "Delete" })] })) : (_jsx("div", { children: "Select a note" })) })] }));
}
