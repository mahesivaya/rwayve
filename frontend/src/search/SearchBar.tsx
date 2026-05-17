import { useLocation } from "react-router-dom";
import { useGlobalSearch } from "./SearchContext";
import { getSearchLabel, shouldHideSearch } from "./SearchConfig";

export default function SearchBar() {
  const location = useLocation();
  const { searchQuery, setSearchQuery } = useGlobalSearch();

  if (shouldHideSearch(location.pathname)) {
    return null;
  }

  const searchLabel = getSearchLabel(location.pathname);

  return (
    <div className="global-search-row">
      <div className="global-search-box">
        <span className="global-search-icon" aria-hidden="true">⌕</span>
        <input
          type="search"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder={`Search ${searchLabel}`}
          aria-label={`Search ${searchLabel}`}
        />
        {searchQuery && (
          <button
            type="button"
            className="global-search-clear"
            onClick={() => setSearchQuery("")}
            title="Clear search"
            aria-label="Clear search"
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
}