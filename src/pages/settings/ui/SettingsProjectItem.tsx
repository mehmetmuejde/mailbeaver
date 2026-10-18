import { Pencil, Trash2 } from "lucide-react";
import type { ProjectOverview } from "@/shared/api";
import { pluralize } from "@/shared/lib/format";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { useDeleteProject } from "../api/use-delete-project";
import { RULE_KIND_LABELS } from "../model/project-form";

type SettingsProjectItemProps = {
  project: ProjectOverview["projects"][number];
  canEdit: boolean;
  onEdit: () => void;
};

export function SettingsProjectItem({ project, canEdit, onEdit }: SettingsProjectItemProps) {
  const remove = useDeleteProject();
  const headingId = `project-${project.id}`;

  return (
    <article aria-labelledby={headingId} className="grid gap-2 rounded-lg border p-3">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 id={headingId} className="font-semibold">
            {project.name}
          </h3>
          <p className="text-xs text-muted-foreground">
            {pluralize(project.total, "message")}, {project.unread} unread
          </p>
        </div>
        <div className="flex gap-2">
          <Button size="sm" icon={<Pencil />} disabled={!canEdit} onClick={onEdit}>
            Edit
          </Button>
          <ConfirmDialog
            trigger={
              <Button
                size="sm"
                variant="destructive"
                icon={<Trash2 />}
                disabled={!canEdit || remove.isPending}
              >
                Delete
              </Button>
            }
            title={`Delete project “${project.name}”?`}
            description="Its messages stay where they are and lose the project."
            confirmLabel="Delete project"
            confirmIcon={<Trash2 />}
            onConfirm={() => remove.mutate(project.id)}
          />
        </div>
      </div>
      {project.rules.length === 0 ? (
        <p className="text-xs text-muted-foreground">No rules: assign mail by adding one.</p>
      ) : (
        <ul className="text-xs">
          {project.rules.map((rule) => (
            <li key={`${rule.kind}:${rule.value}`}>
              <span className="text-muted-foreground">{RULE_KIND_LABELS[rule.kind]}:</span>{" "}
              <code className="font-mono">{rule.value}</code>
            </li>
          ))}
        </ul>
      )}
      {remove.isError ? <p role="alert">The project could not be deleted.</p> : null}
    </article>
  );
}
