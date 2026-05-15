export type AccountType = "personal" | "business_admin" | "project_admin";

export function normalizeAccountType(accountType?: string | null): AccountType {
  if (accountType === "business" || accountType === "business_admin") {
    return "business_admin";
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
