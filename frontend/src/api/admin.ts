import { apiFetch } from "./client";

export type AdminCreatedUser = {
  id: number;
  username: string | null;
  email: string;
  account_type: string;
};

export async function createAdminUser(
  username: string,
  email: string,
  password: string
): Promise<AdminCreatedUser> {
  const res = await apiFetch("/api/admin/users", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({ username, email, password }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to create user");
  }

  return data;
}
