import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useState } from "react";
import "./notes.css";
import { apiFetch } from "../api/client";
export default function Notes() {
    const [notes, setNotes] = useState([]);
    const [selected, setSelected] = useState(null);
    // ================= FETCH NOTES =================
    useEffect(() => {
        const loadNotes = async () => {
            try {
                const res = await apiFetch("/api/notes");
                const data = await res.json();
                setNotes(data);
            }
            catch (err) {
                console.error(err);
            }
        };
        void loadNotes();
    }, []);
    // ================= CREATE =================
    const createNote = async () => {
        try {
            const res = await apiFetch("/api/notes", {
                method: "POST",
                body: JSON.stringify({
                    title: "",
                    content: "",
                }),
            });
            const newNote = await res.json();
            setNotes((prev) => [
                newNote,
                ...prev,
            ]);
            setSelected(newNote);
        }
        catch (err) {
            console.error(err);
        }
    };
    // ================= UPDATE (LOCAL STATE) =================
    const handleChange = (value) => {
        if (!selected) {
            return;
        }
        const updated = { ...selected, content: value };
        setSelected(updated);
        setNotes((prev) => prev.map((n) => n.id === updated.id
            ? updated
            : n));
    };
    // ================= AUTOSAVE =================
    useEffect(() => {
        if (!selected) {
            return;
        }
        const timeout = setTimeout(() => { void saveNote(selected); }, 800);
        return () => clearTimeout(timeout);
    }, [selected]);
    const saveNote = async (note) => {
        try {
            await apiFetch(`/api/notes/${note.id}`, {
                method: "PUT",
                body: JSON.stringify(note),
            });
        }
        catch (err) {
            console.error(err);
        }
    };
    // ================= DELETE =================
    const deleteNote = async (id) => {
        try {
            await apiFetch(`/api/notes/${id}`, {
                method: "DELETE",
            });
            setNotes((prev) => prev.filter((n) => n.id !== id));
            if (selected?.id === id) {
                setSelected(null);
            }
        }
        catch (err) {
            console.error(err);
        }
    };
    return (_jsxs("div", { className: "notes-container", children: [_jsxs("div", { className: "notes-sidebar", children: [_jsx("button", { onClick: () => void createNote(), children: "+ New" }), notes.map((n) => (_jsx("div", { onClick: () => setSelected(n), children: n.title ||
                            "Untitled" }, n.id)))] }), _jsx("div", { className: "notes-editor", children: selected ? (_jsxs(_Fragment, { children: [_jsx("input", { placeholder: "Title", value: selected.title, onChange: (e) => setSelected({
                                ...selected,
                                title: e.target.value,
                            }) }), _jsx("textarea", { value: selected.content, onChange: (e) => handleChange(e.target.value) }), _jsx("button", { onClick: () => void deleteNote(selected.id), children: "Delete" })] })) : (_jsx("div", { children: "Select a note" })) })] }));
}
