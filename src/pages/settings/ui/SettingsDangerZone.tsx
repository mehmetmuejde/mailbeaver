import { useQuery } from "@tanstack/react-query";
import { Trash2 } from "lucide-react";
import { messageQueries } from "@/shared/api";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { useDeleteAllMessages } from "../api/use-delete-all-messages";
import { SettingsReset } from "./SettingsReset";

export function SettingsDangerZone() {
  const { data: counts } = useQuery(messageQueries.folders());
  const deleteAll = useDeleteAllMessages();
  const total =
    counts === undefined ? 0 : counts.inbox.total + counts.archive.total + counts.trash.total;

  return (
    <section
      aria-labelledby="danger-heading"
      className="grid gap-3 rounded-lg border border-destructive/40 p-4"
    >
      <h2 id="danger-heading" className="text-base font-semibold text-destructive">
        Danger zone
      </h2>
      <div className="grid gap-2">
        <h3 className="font-semibold">Delete all messages</h3>
        <p>Deletes every message in every folder. Projects and settings stay.</p>
        <div>
          <ConfirmDialog
            trigger={
              <Button
                variant="destructive"
                icon={<Trash2 />}
                disabled={total === 0 || deleteAll.isPending}
              >
                Delete all messages
              </Button>
            }
            title="Delete all messages?"
            description={`All ${total} messages in Inbox, Archive and Trash will be deleted permanently.`}
            confirmLabel="Delete all"
            confirmIcon={<Trash2 />}
            onConfirm={() => deleteAll.mutate()}
          />
        </div>
        <p role="status" className="text-muted-foreground">
          {deleteAll.isSuccess ? "All messages were deleted." : null}
        </p>
        {deleteAll.isError ? <p role="alert">Messages could not be deleted.</p> : null}
      </div>
      <hr className="border-destructive/20" />
      <SettingsReset />
    </section>
  );
}
