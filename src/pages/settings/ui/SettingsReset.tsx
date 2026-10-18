import { RotateCcw } from "lucide-react";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { useResetAllData } from "../api/use-reset-all-data";

export function SettingsReset() {
  const reset = useResetAllData();
  return (
    <div className="grid gap-2">
      <h3 className="font-semibold">Reset Mailbeaver</h3>
      <p>Deletes all messages, projects and settings. Mailbeaver starts over like new.</p>
      <div>
        <ConfirmDialog
          trigger={
            <Button variant="destructive" icon={<RotateCcw />} disabled={reset.isPending}>
              Reset Mailbeaver
            </Button>
          }
          title="Reset Mailbeaver?"
          description="All messages, projects and settings will be deleted permanently. This cannot be undone."
          confirmLabel="Reset everything"
          confirmIcon={<RotateCcw />}
          onConfirm={() => reset.mutate()}
        />
      </div>
      <p role="status" className="text-muted-foreground">
        {reset.isSuccess ? "Mailbeaver was reset." : null}
      </p>
      {reset.isError ? <p role="alert">Mailbeaver could not be reset.</p> : null}
    </div>
  );
}
