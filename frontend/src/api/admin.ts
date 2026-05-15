import { apiFetch } from "./client";

export type AdminCreatedUser = {
  id: number;
  username: string | null;
  email: string;
  account_type: string;
  organization_id?: number | null;
};

export type AdminOrganization = {
  id: number;
  name: string;
  user_count: number;
};

export async function listAdminOrganizations(): Promise<AdminOrganization[]> {
  const res = await apiFetch("/api/admin/organizations", {
    preserve401: true,
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to load businesses");
  }

  return data;
}

export async function createAdminOrganization(name: string): Promise<AdminOrganization> {
  const res = await apiFetch("/api/admin/organizations", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({ name }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to create business");
  }

  return data;
}

export async function createAdminUser(
  username: string,
  email: string,
  password: string,
  accountType = "personal",
  organizationName = ""
): Promise<AdminCreatedUser> {
  const res = await apiFetch("/api/admin/users", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({
      username,
      email,
      password,
      account_type: accountType,
      organization_name: organizationName,
    }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to create user");
  }

  return data;
}
