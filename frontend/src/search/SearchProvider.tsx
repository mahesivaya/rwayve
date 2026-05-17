import React, { useState, useMemo } from "react";
import { SearchContext } from "./SearchContext";

interface SearchProviderProps {
  children: React.ReactNode;
}

export default function SearchProvider({ children }: SearchProviderProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const value = useMemo(() => {
    const normalizedSearchQuery = searchQuery.trim().toLowerCase();
    return { searchQuery, normalizedSearchQuery, setSearchQuery };
  }, [searchQuery]);

  return (
    <SearchContext.Provider value={value}>
      {children}
    </SearchContext.Provider>
  );
}