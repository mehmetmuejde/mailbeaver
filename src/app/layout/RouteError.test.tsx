import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RouteError } from "./RouteError";

describe("RouteError", () => {
  it("shows the error and retries", async () => {
    const reset = vi.fn();
    const user = userEvent.setup();
    render(<RouteError error={new Error("boom")} reset={reset} />);

    expect(screen.getByRole("alert")).toHaveTextContent("boom");
    await user.click(screen.getByRole("button", { name: "Try again" }));
    expect(reset).toHaveBeenCalledOnce();
  });

  it("shows thrown values that are not errors", () => {
    render(<RouteError error={"plain text" as unknown as Error} reset={vi.fn()} />);
    expect(screen.getByRole("alert")).toHaveTextContent("plain text");
  });
});
