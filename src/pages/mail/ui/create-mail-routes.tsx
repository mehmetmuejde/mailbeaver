import { useQuery } from "@tanstack/react-query";
import {
  type AnyRoute,
  createRoute,
  Navigate,
  useNavigate,
  useSearch,
} from "@tanstack/react-router";
import { useState } from "react";
import { type Folder, settingsQueries } from "@/shared/api";
import { hasViewParams, viewToSearch } from "../model/folder-view";
import { mailSearchSchema } from "../model/mail-search";
import { MailPage } from "./MailPage";

function createFolderRoute<TParent extends AnyRoute, TPath extends string>(
  parentRoute: TParent,
  path: TPath,
  folder: Folder,
) {
  function FolderRoute() {
    // The route already validated the search; parsing again gives the generic factory
    // its types back, which the parent type parameter erases from the route hooks.
    const search = mailSearchSchema.parse(useSearch({ strict: false }));
    const navigate = useNavigate();
    const views = useQuery(settingsQueries.folderViews());
    // Restore only when the folder is entered; later changes on the page must stay.
    const [restored, setRestored] = useState(false);

    if (!restored) {
      if (views.isPending) {
        return null;
      }
      const saved =
        views.data === undefined || hasViewParams(search) ? {} : viewToSearch(views.data[folder]);
      if (Object.values(saved).some((value) => value !== undefined)) {
        return <Navigate to="." search={(prev) => ({ ...prev, ...saved })} replace />;
      }
      setRestored(true);
    }

    return (
      <MailPage
        folder={folder}
        search={search}
        onSearchChange={(patch, options) =>
          void navigate({
            to: ".",
            search: (prev) => ({ ...prev, ...patch }),
            replace: options?.replace,
          })
        }
      />
    );
  }

  return createRoute({
    getParentRoute: () => parentRoute,
    path,
    validateSearch: mailSearchSchema,
    component: FolderRoute,
  });
}

/** The three folder views share one page; only the folder differs. */
export function createMailRoutes<TParent extends AnyRoute>(parentRoute: TParent) {
  return {
    inbox: createFolderRoute(parentRoute, "/inbox", "inbox"),
    archive: createFolderRoute(parentRoute, "/archive", "archive"),
    trash: createFolderRoute(parentRoute, "/trash", "trash"),
  };
}
