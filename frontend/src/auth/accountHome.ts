export type AccountType = "personal" | "business" | "business_admin" | "project_admin";

export function normalizeAccountType(accountType?: string | null): AccountType {
  if (accountType === "business_admin") {
    return "business_admin";
  }
  
  if (accountType === "business") {
    return "business";
  }

  if (accountType === "project_admin") {
    return "project_admin";
  }

  return "personal";
}

export function homePathForAccount(accountType?: string | null) {
  const normalized = normalizeAccountType(accountType);
  if (normalized === "project_admin") return "/project-admin-home";
  if (normalized === "business_admin") return "/business-home";
  return "/home";
}

type AccountLike = {
  account_type?: string | null;
  organization_id?: number | null;
  organization_slug?: string | null;
};

// Landing route for a fully-resolved user. Business members (business admins
// and the personal accounts created inside a business) land on their own
// /business/<slug> home page. Until the slug is known (optimistic JWT boot or
// right after login, before /api/me resolves) we fall back to "/home", which
// re-redirects itself once the slug arrives — never a dead end.
export function homePathForUser(user?: AccountLike | null): string {
  const normalized = normalizeAccountType(user?.account_type);
  if (normalized === "project_admin") return "/project-admin-home";
  if (normalized === "business_admin") return "/business-home";

  const inBusiness =
    normalized === "business" || user?.organization_id != null;
  if (inBusiness) {
    return user?.organization_slug
      ? `/business/${user.organization_slug}`
      : "/home";
  }

  return "/home";
}
