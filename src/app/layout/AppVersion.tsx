import { useQuery } from "@tanstack/react-query";
import type { MouseEvent } from "react";
import { settingsQueries } from "@/shared/api";
import { isDesktopApp, openExternalUrl } from "@/shared/lib/platform";

export function AppVersion() {
  const { data: info } = useQuery(settingsQueries.info());
  const { data: update } = useQuery(settingsQueries.update());
  const releaseUrl = update?.updateAvailable ? update.url : null;

  // The desktop webview would open the page inside the app; the browser shows it.
  const openRelease = (event: MouseEvent<HTMLAnchorElement>) => {
    if (releaseUrl !== null && isDesktopApp()) {
      event.preventDefault();
      void openExternalUrl(releaseUrl);
    }
  };

  return (
    <div className="grid gap-1 px-4 py-4 text-center text-xs text-muted-foreground">
      <p>{info === undefined ? null : `Version ${info.version}`}</p>
      {releaseUrl === null || update === undefined ? null : (
        <a
          href={releaseUrl}
          target="_blank"
          rel="noreferrer"
          onClick={openRelease}
          className="font-medium text-primary underline underline-offset-2"
        >
          Version {update.latestVersion} available
        </a>
      )}
    </div>
  );
}
