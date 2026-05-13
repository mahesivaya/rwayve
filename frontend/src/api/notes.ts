import { apiFetch } from "./client";

export type Note = {
  id: number;
  title: string | null;
  content: string | null;
  updated_at?: string | null;
};

export type SaveNotePayload = {
  title: string;
  content: string;
};

export const getNotes = async () => {
  const res = await apiFetch("/api/notes");
  return res.json() as Promise<Note[]>;
};

export const createNoteApi = async (payload: SaveNotePayload) => {
  const res = await apiFetch("/api/notes", {
    method: "POST",
    body: JSON.stringify(payload),
  });

  return res.json() as Promise<Note>;
};

export const updateNoteApi = async (
  id: number,
  payload: SaveNotePayload
) => {
  const res = await apiFetch(`/api/notes/${id}`, {
    method: "PUT",
    body: JSON.stringify(payload),
  });

  return res.json() as Promise<Note>;
};

export const deleteNoteApi = async (id: number) => {
  await apiFetch(`/api/notes/${id}`, {
    method: "DELETE",
  });
};
