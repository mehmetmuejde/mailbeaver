import { useQuery } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { useState } from "react";
import { projectQueries } from "@/shared/api";
import { pluralize } from "@/shared/lib/format";
import { Button } from "@/shared/ui/button";
import { EMPTY_PROJECT } from "../model/project-form";
import { SettingsProjectForm } from "./SettingsProjectForm";
import { SettingsProjectItem } from "./SettingsProjectItem";

/** `"new"` while a project is being created, a project id while it is edited. */
type Editing = number | "new" | null;

export function SettingsProjects() {
  const { data, isPending, isError } = useQuery(projectQueries.overview());
  const [editing, setEditing] = useState<Editing>(null);
  const close = () => setEditing(null);

  return (
    <section aria-labelledby="projects-heading" className="grid gap-3">
      <div className="flex items-center justify-between gap-4">
        <h2 id="projects-heading" className="text-base font-semibold">
          Projects
        </h2>
        <Button
          size="sm"
          icon={<Plus />}
          disabled={editing !== null}
          onClick={() => setEditing("new")}
        >
          New project
        </Button>
      </div>
      <p className="text-muted-foreground">
        Rules sort incoming mail into projects. Mail sent with SMTP credentials gets a project named
        after the username automatically.
      </p>
      {editing === "new" ? (
        <SettingsProjectForm projectId={null} initial={EMPTY_PROJECT} onDone={close} />
      ) : null}
      {isPending ? <p role="status">Loading projects…</p> : null}
      {isError ? <p role="alert">Projects could not be loaded.</p> : null}
      {data === undefined ? null : (
        <>
          {data.projects.length === 0 && editing !== "new" ? (
            <p className="text-muted-foreground">No projects yet.</p>
          ) : null}
          <ul className="grid gap-2">
            {data.projects.map((project) => (
              <li key={project.id}>
                {editing === project.id ? (
                  <SettingsProjectForm
                    projectId={project.id}
                    initial={{ name: project.name, rules: project.rules, notify: project.notify }}
                    onDone={close}
                  />
                ) : (
                  <SettingsProjectItem
                    project={project}
                    canEdit={editing === null}
                    onEdit={() => setEditing(project.id)}
                  />
                )}
              </li>
            ))}
          </ul>
          <p className="text-muted-foreground">
            {pluralize(data.unassigned.total, "message")} without a project.
          </p>
        </>
      )}
    </section>
  );
}
