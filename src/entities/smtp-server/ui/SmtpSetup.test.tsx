import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQuery } from "@/shared/lib/test";
import { SmtpSetup } from "./SmtpSetup";

describe("SmtpSetup", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("shows host and port and copies them", async () => {
    const user = userEvent.setup();
    const writeText = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined);
    renderWithQuery(<SmtpSetup address="127.0.0.1:1026" />);

    expect(screen.getByText("127.0.0.1")).toBeInTheDocument();
    expect(screen.getByText("1026")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Copy port" }));
    expect(writeText).toHaveBeenLastCalledWith("1026");
    expect(await screen.findByText("Copied.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Copy as environment variables" }));
    expect(writeText).toHaveBeenLastCalledWith("SMTP_HOST=127.0.0.1\nSMTP_PORT=1026");
  });

  it("reports a failed copy", async () => {
    const user = userEvent.setup();
    vi.spyOn(navigator.clipboard, "writeText").mockRejectedValue(new Error("denied"));
    renderWithQuery(<SmtpSetup address="127.0.0.1:1026" />);

    await user.click(screen.getByRole("button", { name: "Copy host" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Copying failed.");
  });
});
