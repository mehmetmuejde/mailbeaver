import { useState } from "react";

type CopyResult = { kind: "copied"; value: string } | { kind: "failed" } | null;

/** Copies text and remembers the outcome, so the UI can confirm it or report a failure. */
export function useCopyToClipboard() {
  const [result, setResult] = useState<CopyResult>(null);
  const copy = (value: string) => {
    navigator.clipboard.writeText(value).then(
      () => setResult({ kind: "copied", value }),
      () => setResult({ kind: "failed" }),
    );
  };
  return { copy, result };
}
