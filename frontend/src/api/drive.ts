import { API_BASE } from "../config/env";
import { getAuthToken } from "../auth/token";
import { apiFetch } from "./client";

export type UploadedFile = {
  id: number;
  name: string;
  file_type: string;
  size: number;
  drive_url?: string;
  created_at: string;
};

export const getDriveFiles = async (userId: number) => {
  const params = new URLSearchParams({ user_id: String(userId) });
  const res = await apiFetch(`/api/files?${params.toString()}`);
  return res.json() as Promise<UploadedFile[]>;
};

export const uploadDriveFiles = async (userId: number, files: File[]) => {
  const formData = new FormData();
  formData.append("user_id", userId.toString());
  files.forEach((file) => formData.append("files", file));

  const token = getAuthToken();
  const res = await fetch(`${API_BASE}/api/files/upload`, {
    method: "POST",
    body: formData,
    headers: token
      ? {
          Authorization: `Bearer ${token}`,
        }
      : undefined,
  });

  if (!res.ok) {
    throw new Error("Upload failed");
  }
};
