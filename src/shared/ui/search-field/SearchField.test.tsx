import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SearchField } from "./SearchField";

describe("SearchField", () => {
  it("is a labelled search box", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<SearchField label="Search messages" value="" onChange={onChange} />);

    await user.type(screen.getByRole("searchbox", { name: "Search messages" }), "a");
    expect(onChange).toHaveBeenCalledWith("a");
  });
});
