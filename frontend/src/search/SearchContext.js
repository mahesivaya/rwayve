import { createContext, useContext } from "react";
export const SearchContext = createContext(null);
export function useGlobalSearch() {
    const value = useContext(SearchContext);
    if (!value) {
        return {
            searchQuery: "",
            normalizedSearchQuery: "",
            setSearchQuery: () => { },
        };
    }
    return value;
}
