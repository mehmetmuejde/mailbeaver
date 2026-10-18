import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { isDesktopApp, openExternalUrl } from "@/shared/lib/platform";
import { MailAttachmentList } from "./MailAttachmentList";

vi.mock("@/shared/lib/platform", () => ({
  isDesktopApp: vi.fn(),
  openExternalUrl: vi.fn(),
}));

const attachments = [
  { index: 0, filename: "invoice.pdf", contentType: "application/pdf", size: 2048 },
];

describe("MailAttachmentList", () => {
  afterEach(() => {
    vi.resetAllMocks();
  });

  it("lists attachments with type and size", () => {
    render(<MailAttachmentList messageId={7} attachments={attachments} />);
    expect(screen.getByRole("link", { name: "invoice.pdf" })).toHaveAttribute(
      "href",
      "/api/messages/7/attachments/0",
    );
    expect(screen.getByText("application/pdf, 2.0 KB")).toBeInTheDocument();
  });

  it("opens attachments in the system browser in the desktop app", async () => {
    vi.mocked(isDesktopApp).mockReturnValue(true);
    vi.mocked(openExternalUrl).mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(<MailAttachmentList messageId={7} attachments={attachments} />);

    await user.click(screen.getByRole("link", { name: "invoice.pdf" }));

    expect(openExternalUrl).toHaveBeenCalledWith("/api/messages/7/attachments/0");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("reports when the attachment cannot be opened", async () => {
    vi.mocked(isDesktopApp).mockReturnValue(true);
    vi.mocked(openExternalUrl).mockRejectedValue(new Error("not allowed"));
    const user = userEvent.setup();
    render(<MailAttachmentList messageId={7} attachments={attachments} />);

    await user.click(screen.getByRole("link", { name: "invoice.pdf" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The attachment could not be opened.",
    );
  });
});
