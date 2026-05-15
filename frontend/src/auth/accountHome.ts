export type AccountType = "personal" | "business";

export function homePathForAccount(accountType?: string | null) {
  return accountType === "business" ? "/business-home" : "/home";
}
