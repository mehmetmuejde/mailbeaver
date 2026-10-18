import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Archive } from "lucide-react";
import { describe, expect, it, vi } from "vitest";
import { TooltipRoot } from "@/shared/ui/tooltip";
import { IconButton } from "./IconButton";

describe("IconButton", () => {
  it("is named by its label and shows it as a tooltip on focus", async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(
      <TooltipRoot>
        <IconButton label="Archive" icon={<Archive />} onClick={onClick} />
      </TooltipRoot>,
    );

    await user.tab();
    expect(screen.getByRole("button", { name: "Archive" })).toHaveFocus();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("Archive");

    await user.keyboard("{Enter}");
    expect(onClick).toHaveBeenCalledOnce();
  });
});
