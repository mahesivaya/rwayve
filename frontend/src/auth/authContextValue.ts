import { createContext } from "react";
import type { AccountType } from "./accountHome";

export type UserType = {
  email: string;
  id: number;
  account_type: AccountType;
  organization_id?: number | null;
  organization_slug?: string | null;
  organization_name?: string | null;
};

export type AuthType = {
  user: UserType | null;
  initializing: boolean;
  login: (token: string, accountType?: string) => void;
  logout: () => void;
};

export const AuthContext = createContext<AuthType | null>(null);
