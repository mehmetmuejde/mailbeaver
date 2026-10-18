import { useQuery } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import { messageQueries } from "@/shared/api";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { useEmptyTrash } from "../api/use-empty-trash";

type MailEmptyTrashProps = {
  onEmptied: () => void;
};

export function MailEmptyTrash({ onEmptied }: MailEmptyTrashProps) {
  const { data: counts } = useQuery(messageQueries.folders());
  const emptyTrash = useEmptyTrash();
  const total = counts?.trash.total ?? 0;

  return (
    <div className="grid gap-1">
      <ConfirmDialog
        trigger={
          <Button
            variant="destructive"
            icon={<Trash2 />}
            className="w-full"
            disabled={total === 0 || emptyTrash.isPending}
          >
            Empty trash
          </Button>
        }
        title="Empty trash?"
        description={`${total} messages will be deleted permanently. This cannot be undone.`}
        confirmLabel="Empty trash"
        confirmIcon={<Trash2 />}
        onConfirm={() => emptyTrash.mutate(undefined, { onSuccess: onEmptied })}
      />
      {emptyTrash.isError ? <p role="alert">Trash could not be emptied.</p> : null}
    </div>
  );
}
