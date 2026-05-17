import { createContext, useContext } from "react";

export type SearchContextValue = {
  searchQuery: string;
  normalizedSearchQuery: string;
  setSearchQuery: (value: string) => void;
  emailViewLayout: "list" | "split";
  setEmailViewLayout: (value: "list" | "split") => void;
};

export const SearchContext = createContext<SearchContextValue | null>(null);

export function useGlobalSearch() {
  const value = useContext(SearchContext);

  if (!value) {
    return {
      searchQuery: "",
      normalizedSearchQuery: "",
      setSearchQuery: () => {},
      emailViewLayout: "split",
      setEmailViewLayout: () => {},
    };
  }

  return value;
}
