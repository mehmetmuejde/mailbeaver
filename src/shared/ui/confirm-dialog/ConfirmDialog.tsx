import { X } from "lucide-react";
import type { ReactNode } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/shared/ui/primitives/alert-dialog";

type ConfirmDialogProps = {
  /** The button that opens the dialog; focus returns to it on close. */
  trigger: ReactNode;
  title: string;
  description: ReactNode;
  confirmLabel: string;
  confirmIcon: ReactNode;
  onConfirm: () => void;
};

/**
 * Asks before an irreversible action. Focus starts on Cancel, Escape closes the dialog.
 */
export function ConfirmDialog({
  trigger,
  title,
  description,
  confirmLabel,
  confirmIcon,
  onConfirm,
}: ConfirmDialogProps) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>{trigger}</AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>
            <span data-icon="inline-start" aria-hidden="true" className="inline-flex">
              <X />
            </span>
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            <span data-icon="inline-start" aria-hidden="true" className="inline-flex">
              {confirmIcon}
            </span>
            {confirmLabel}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
