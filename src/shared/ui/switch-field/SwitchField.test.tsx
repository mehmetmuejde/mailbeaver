import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SwitchField } from "./SwitchField";

describe("SwitchField", () => {
  it("toggles with its label and is described", async () => {
    const onCheckedChange = vi.fn();
    render(
      <SwitchField
        label="Play a sound"
        description="Uses the system sound."
        checked={false}
        onCheckedChange={onCheckedChange}
      />,
    );

    const toggle = screen.getByRole("switch", { name: "Play a sound" });
    expect(toggle).toHaveAccessibleDescription("Uses the system sound.");
    await userEvent.click(screen.getByText("Play a sound"));
    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });
});
