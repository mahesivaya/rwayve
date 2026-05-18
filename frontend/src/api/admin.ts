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

// The organization admin to provision alongside the new organization. The
// caller supplies the full login email, built from the admin handle and the
// organization slug as <adminHandle>@<org-slug>.com.
export type CreateOrganizationInput = {
  name: string;
  adminUsername: string;
  adminEmail: string;
  adminPassword: string;
};

export async function listAdminOrganizations(): Promise<AdminOrganization[]> {
  const res = await apiFetch("/api/admin/organizations", {
    preserve401: true,
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to load organizations");
  }

  return data;
}

export async function createAdminOrganization(
  input: CreateOrganizationInput
): Promise<AdminOrganization> {
  const res = await apiFetch("/api/admin/organizations", {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({
      name: input.name,
      admin_username: input.adminUsername,
      admin_email: input.adminEmail,
      admin_password: input.adminPassword,
    }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to create organization");
  }

  return data;
}

// A stored API key as shown in the admin UI. `key_preview` is redacted; the
// raw key is returned only once, by generateOrganizationApiKey.
export type ApiKey = {
  id: number;
  name: string;
  key_preview: string;
  created_at: string;
  last_used_at: string | null;
  revoked_at: string | null;
};

export type GeneratedApiKey = {
  id: number;
  name: string;
  key_preview: string;
  created_at: string;
  api_key: string;
};

export async function generateOrganizationApiKey(
  organizationId: number,
  name: string
): Promise<GeneratedApiKey> {
  const res = await apiFetch(`/api/admin/organizations/${organizationId}/keys`, {
    method: "POST",
    preserve401: true,
    body: JSON.stringify({ name }),
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to generate API key");
  }

  return data;
}

export async function listOrganizationApiKeys(
  organizationId: number
): Promise<ApiKey[]> {
  const res = await apiFetch(`/api/admin/organizations/${organizationId}/keys`, {
    preserve401: true,
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message || "Failed to load API keys");
  }

  return data;
}

export async function revokeOrganizationApiKey(
  organizationId: number,
  keyId: number
): Promise<void> {
  await apiFetch(
    `/api/admin/organizations/${organizationId}/keys/${keyId}`,
    { method: "DELETE", preserve401: true }
  );
}

// Creates a user as the calling admin. `email` is the full login address; the
// caller builds it from a handle and the organization domain (or wayve.com for
// personal accounts).
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
