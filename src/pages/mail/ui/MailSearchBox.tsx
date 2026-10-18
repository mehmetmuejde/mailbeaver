import { useEffect, useRef, useState } from "react";
import { SEARCH_DEBOUNCE_MS } from "@/shared/config";
import { SearchField } from "@/shared/ui/search-field";
import { type MailSearchChange, SEARCH_MAX_LENGTH } from "../model/mail-search";
import { MailSearchHelp } from "./MailSearchHelp";

type MailSearchBoxProps = {
  initialQuery: string;
  onSearchChange: MailSearchChange;
};

/** Searches all folders after a pause in typing; Escape ends the search. */
export function MailSearchBox({ initialQuery, onSearchChange }: MailSearchBoxProps) {
  const [query, setQuery] = useState(initialQuery);
  const timer = useRef<number | undefined>(undefined);

  // A pending search must not fire after the page is gone.
  useEffect(() => () => window.clearTimeout(timer.current), []);

  const handleChange = (value: string) => {
    setQuery(value);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      const q = value.trim() === "" ? undefined : value;
      onSearchChange(
        { q, folder: undefined, message: undefined, page: undefined },
        { replace: true },
      );
    }, SEARCH_DEBOUNCE_MS);
  };

  const handleEscape = () => {
    window.clearTimeout(timer.current);
    setQuery("");
    onSearchChange(
      { q: undefined, folder: undefined, message: undefined, page: undefined },
      { replace: true },
    );
  };

  return (
    <search className="ml-auto flex w-full max-w-xs items-center gap-1">
      <SearchField
        label="Search all mail"
        placeholder="Search all mail…"
        value={query}
        maxLength={SEARCH_MAX_LENGTH}
        onChange={handleChange}
        onEscape={handleEscape}
      />
      <MailSearchHelp />
    </search>
  );
}
