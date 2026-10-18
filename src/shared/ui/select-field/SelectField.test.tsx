import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SelectField } from "./SelectField";

describe("SelectField", () => {
  it("is labelled and reports the chosen value", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(
      <SelectField
        label="Project"
        hideLabel
        options={[
          { value: "*", label: "All projects" },
          { value: "=shop", label: "shop" },
        ]}
        value="*"
        onChange={onChange}
      />,
    );

    await user.selectOptions(screen.getByRole("combobox", { name: "Project" }), "shop");
    expect(onChange).toHaveBeenCalledWith("=shop");
  });
});
