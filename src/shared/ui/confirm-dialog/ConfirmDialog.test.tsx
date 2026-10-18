import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Trash2 } from "lucide-react";
import { describe, expect, it, vi } from "vitest";
import { Button } from "@/shared/ui/button";
import { ConfirmDialog } from "./ConfirmDialog";

function renderDialog(onConfirm: () => void) {
  render(
    <ConfirmDialog
      trigger={<Button icon={<Trash2 />}>Empty trash</Button>}
      title="Empty trash?"
      description="3 messages will be deleted permanently."
      confirmLabel="Empty trash"
      confirmIcon={<Trash2 />}
      onConfirm={onConfirm}
    />,
  );
}

describe("ConfirmDialog", () => {
  it("starts on Cancel and closes without confirming", async () => {
    const onConfirm = vi.fn();
    const user = userEvent.setup();
    renderDialog(onConfirm);

    await user.click(screen.getByRole("button", { name: "Empty trash" }));
    const dialog = screen.getByRole("alertdialog", { name: "Empty trash?" });
    expect(dialog).toHaveAccessibleDescription("3 messages will be deleted permanently.");
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Empty trash" })).toHaveFocus();
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("runs the action on confirmation", async () => {
    const onConfirm = vi.fn();
    const user = userEvent.setup();
    renderDialog(onConfirm);

    await user.click(screen.getByRole("button", { name: "Empty trash" }));
    await user.click(screen.getByRole("button", { name: "Empty trash" }));

    expect(onConfirm).toHaveBeenCalledOnce();
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });
});
