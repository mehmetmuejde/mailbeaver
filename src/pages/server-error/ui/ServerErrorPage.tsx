import type { ReactNode } from "react";
import { isDesktopApp } from "@/shared/lib/platform";
import { ServerErrorPortForm } from "./ServerErrorPortForm";

type ServerErrorPageProps = {
  message: string;
};

function isPortConflict(message: string): boolean {
  return /port \d+ is not available/.test(message);
}

/** Advice for the startup errors the core reports; none for the rest. */
function hintFor(message: string): ReactNode {
  if (isPortConflict(message)) {
    return isDesktopApp() ? (
      "Another program already uses the port. Choose other ports; Mailbeaver restarts with them."
    ) : (
      <>
        Another program already uses the port. Set{" "}
        <code className="font-mono">MAILBEAVER_SMTP_PORT</code> or{" "}
        <code className="font-mono">MAILBEAVER_HTTP_PORT</code> to a free port and restart.
      </>
    );
  }
  if (message.includes("schema version")) {
    return (
      <>
        The database was written by a newer Mailbeaver. Update the app, or move the database file
        named above aside to start with an empty one.
      </>
    );
  }
  return null;
}

export function ServerErrorPage({ message }: ServerErrorPageProps) {
  const hint = hintFor(message);
  return (
    <main className="mx-auto mt-[10vh] grid max-w-2xl gap-3 p-4">
      {/* The window has no title bar of its own: the top strip moves it. */}
      <div data-tauri-drag-region className="fixed inset-x-0 top-0 h-14" />
      <h1 className="text-lg font-semibold">Mailbeaver could not start</h1>
      <p role="alert">
        <code className="font-mono">{message}</code>
      </p>
      {hint === null ? null : <p>{hint}</p>}
      {isPortConflict(message) && isDesktopApp() ? <ServerErrorPortForm /> : null}
    </main>
  );
}
