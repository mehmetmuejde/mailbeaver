import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ServerErrorPage } from "./ServerErrorPage";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => undefined),
  isTauri: vi.fn(() => true),
}));

describe("ServerErrorPage in the desktop app", () => {
  it("saves other ports and restarts after a port conflict", async () => {
    const user = userEvent.setup();
    render(<ServerErrorPage message="SMTP port 1026 is not available" />);
    const smtp = screen.getByRole("textbox", { name: "SMTP port" });

    await user.clear(smtp);
    await user.type(smtp, "8026");
    await user.click(screen.getByRole("button", { name: "Save and restart" }));
    expect(await screen.findByText("SMTP and HTTP need different ports")).toBeInTheDocument();

    await user.clear(smtp);
    await user.type(smtp, "2526");
    await user.click(screen.getByRole("button", { name: "Save and restart" }));
    expect(invoke).toHaveBeenCalledWith("save_server_ports", { smtpPort: 2526, httpPort: 8026 });
    expect(invoke).toHaveBeenCalledWith("restart_app");
  });

  it("shows why saving failed", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("the ports are set by MAILBEAVER_SMTP_PORT"));
    const user = userEvent.setup();
    render(<ServerErrorPage message="HTTP port 8026 is not available" />);
    await user.click(screen.getByRole("button", { name: "Save and restart" }));
    expect(
      await screen.findByText("the ports are set by MAILBEAVER_SMTP_PORT"),
    ).toBeInTheDocument();
  });

  it("offers no form for other startup errors", () => {
    render(<ServerErrorPage message="cannot open database /x.db: disk full" />);
    expect(screen.queryByRole("form", { name: "Choose other ports" })).not.toBeInTheDocument();
  });
});
