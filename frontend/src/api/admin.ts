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
  slug?: string | null;
  user_count: number;
  admin?: AdminCreatedUser | null;
};

// The business admin's email is generated server-side as
// <adminHandle>@<business-slug>.com — only the handle is supplied here.
export type CreateBusinessInput = {
  name: string;
  adminHandle: string;
  adminPassword: string;
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

export async function createAdminOrganization(
  input: CreateBusinessInput
): Promise<AdminOrganization> {
  const res = await apiFetch("/api/admin/organizations", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({
      name: input.name,
      admin_handle: input.adminHandle,
      admin_password: input.adminPassword,
    }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to create organization");
  }

  return data;
}

// `handle` is the email local part; the backend builds the full address using
// the business domain (or wayve.com for personal accounts).
export async function createAdminUser(
  handle: string,
  password: string,
  accountType = "personal",
  organizationName = ""
): Promise<AdminCreatedUser> {
  const res = await apiFetch("/api/admin/users", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({
      handle,
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
