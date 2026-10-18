import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mailFixture, renderWithQuery } from "@/shared/lib/test";
import { MailMessageView } from "./MailMessageView";

const textOnly = mailFixture({
  id: 2,
  from: "",
  to: [],
  subject: "",
  isRead: true,
  envelopeFrom: "bounce@example.com",
  envelopeTo: ["rcpt@example.com"],
  headers: [],
  text: "Only text",
  html: null,
  project: { id: 1, name: "shop" },
  authUser: "shop-app",
});

function stubFetch(handler: (url: string, method: string) => Response) {
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) =>
    handler(String(input), init?.method ?? "GET"),
  );
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

describe("MailMessageView", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("falls back to the envelope and opens on the text tab without HTML", async () => {
    stubFetch(() => Response.json(textOnly));
    renderWithQuery(<MailMessageView messageId={2} onClose={vi.fn()} />);

    expect(await screen.findByRole("heading", { name: "(no subject)" })).toBeVisible();
    expect(screen.getByText("bounce@example.com")).toBeInTheDocument();
    expect(screen.getByText("rcpt@example.com")).toBeInTheDocument();
    expect(screen.getByText("shop")).toBeInTheDocument();
    expect(screen.getByText("shop-app")).toBeInTheDocument();
    expect(screen.queryByRole("tab", { name: "HTML" })).not.toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Text" })).toHaveAttribute("aria-selected", "true");
    expect(screen.queryByRole("heading", { name: "Attachments" })).not.toBeInTheDocument();
  });

  it("says when the message is gone", async () => {
    stubFetch(() => new Response(null, { status: 404 }));
    renderWithQuery(<MailMessageView messageId={2} onClose={vi.fn()} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("This message no longer exists.");
  });

  it("reports other load failures", async () => {
    stubFetch(() => new Response(null, { status: 500 }));
    renderWithQuery(<MailMessageView messageId={2} onClose={vi.fn()} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("The message could not be loaded.");
  });

  it("keeps the message and reports a failed move", async () => {
    stubFetch((_url, method) =>
      method === "POST" ? new Response(null, { status: 500 }) : Response.json(textOnly),
    );
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderWithQuery(<MailMessageView messageId={2} onClose={onClose} />);

    await user.click(await screen.findByRole("button", { name: "Archive" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("The message could not be moved.");
    expect(onClose).not.toHaveBeenCalled();
  });

  it("reports a failed permanent delete", async () => {
    const trashed = { ...textOnly, folder: "trash" };
    stubFetch((_url, method) =>
      method === "DELETE" ? new Response(null, { status: 500 }) : Response.json(trashed),
    );
    const user = userEvent.setup();
    renderWithQuery(<MailMessageView messageId={2} onClose={vi.fn()} />);

    await user.click(await screen.findByRole("button", { name: "Delete permanently" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("The message could not be deleted.");
  });

  it("reports when the source cannot be loaded", async () => {
    stubFetch((url) =>
      url.endsWith("/raw") ? new Response(null, { status: 500 }) : Response.json(textOnly),
    );
    const user = userEvent.setup();
    renderWithQuery(<MailMessageView messageId={2} onClose={vi.fn()} />);

    await user.click(await screen.findByRole("tab", { name: "Source" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("The source could not be loaded.");
  });
});
