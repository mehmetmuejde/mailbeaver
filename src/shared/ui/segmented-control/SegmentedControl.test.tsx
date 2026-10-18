import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SegmentedControl } from "./SegmentedControl";

const options = [
  { value: "desktop", label: "Desktop" },
  { value: "mobile", label: "Mobile" },
] as const;

describe("SegmentedControl", () => {
  it("marks the current option as pressed and reports a new choice", async () => {
    const onValueChange = vi.fn();
    const user = userEvent.setup();
    render(
      <SegmentedControl
        label="Preview width"
        options={options}
        value="desktop"
        onValueChange={onValueChange}
      />,
    );

    expect(screen.getByRole("radiogroup", { name: "Preview width" })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Desktop" })).toBeChecked();

    await user.click(screen.getByRole("radio", { name: "Mobile" }));
    expect(onValueChange).toHaveBeenCalledWith("mobile");
  });

  it("keeps one option selected when the active one is pressed again", async () => {
    const onValueChange = vi.fn();
    const user = userEvent.setup();
    render(
      <SegmentedControl
        label="Preview width"
        options={options}
        value="desktop"
        onValueChange={onValueChange}
      />,
    );

    await user.click(screen.getByRole("radio", { name: "Desktop" }));
    expect(onValueChange).not.toHaveBeenCalled();
  });
});
