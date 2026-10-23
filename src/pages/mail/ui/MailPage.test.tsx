import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Folder, MessageDetail } from "@/shared/api";
import { createFakeMailApi, mailFixture, renderWithQuery } from "@/shared/lib/test";
import type { MailSearch } from "../model/mail-search";
import { MailPage } from "./MailPage";

/** Stands in for the router: keeps the URL state in memory. */
function Harness({ folder, initial = {} }: { folder: Folder; initial?: MailSearch }) {
  const [search, setSearch] = useState<MailSearch>(initial);
  return (
    <MailPage
      folder={folder}
      search={search}
      onSearchChange={(patch) => setSearch((previous) => ({ ...previous, ...patch }))}
    />
  );
}

const welcome = mailFixture({
  id: 1,
  subject: "Welcome",
  hasAttachments: true,
  html: "<p>Hello <b>Bob</b></p>",
  attachments: [{ index: 0, filename: "invoice.pdf", contentType: "application/pdf", size: 10 }],
});

let api: ReturnType<typeof createFakeMailApi>;

function setup(messages: MessageDetail[], folder: Folder = "inbox", initial?: MailSearch) {
  api = createFakeMailApi(messages);
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<Harness folder={folder} initial={initial} />);
  return user;
}

const list = () => within(screen.getByRole("region", { name: "Message list" }));

/** Count labels split number and word into two elements. */
const isBadge = (text: string) => (_: string, element: Element | null) =>
  element?.getAttribute("data-slot") === "badge" && element.textContent === text;
const badge = (text: string) => screen.getByText(isBadge(text));
const queryBadge = (text: string) => screen.queryByText(isBadge(text));

describe("MailPage", () => {
  beforeEach(() => {
    api = createFakeMailApi([]);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("lists messages and opens one, marking it read", async () => {
    const user = setup([welcome]);

    const item = await list().findByRole("button", { name: /Unread:\s*Alice.*Welcome/ });
    expect(screen.getByRole("heading", { level: 1, name: "Inbox" })).toBeInTheDocument();
    expect(badge("1 message")).toBeInTheDocument();
    expect(badge("1 unread")).toBeInTheDocument();

    await user.click(item);

    expect(await screen.findByRole("heading", { level: 2, name: "Welcome" })).toBeInTheDocument();
    expect(screen.getByTitle("HTML preview")).toHaveAttribute("srcdoc", welcome.html);
    expect(screen.getByRole("link", { name: "invoice.pdf" })).toBeInTheDocument();
    await waitFor(() => expect(api.calls("POST", "/api/messages/1/read")).toHaveLength(1));
    await waitFor(() => expect(queryBadge("1 unread")).not.toBeInTheDocument());
  });

  it("shows text, links, headers and the lazily loaded source in tabs", async () => {
    const user = setup([{ ...welcome, text: "Visit https://shop.example/x" }]);
    await user.click(await list().findByRole("button", { name: /Welcome/ }));
    await screen.findByRole("tab", { name: "HTML" });

    // The device switch belongs to the HTML tab and sits in the same row.
    expect(screen.getByRole("radiogroup", { name: "Preview width" })).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Text" }));
    expect(screen.queryByRole("radiogroup", { name: "Preview width" })).not.toBeInTheDocument();
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Visit https://shop.example/x");

    await user.click(screen.getByRole("tab", { name: "Links" }));
    expect(screen.getByRole("tabpanel")).toHaveTextContent("https://shop.example/x");

    await user.click(screen.getByRole("tab", { name: "Headers" }));
    expect(screen.getByRole("rowheader", { name: "Subject" })).toBeInTheDocument();

    expect(api.calls("GET", "/api/messages/1/raw")).toHaveLength(0);
    await user.click(screen.getByRole("tab", { name: "Source" }));
    expect(await screen.findByText(/Subject: Welcome/)).toBeInTheDocument();
  });

  it("archives the open message without asking and clears the selection", async () => {
    const user = setup([welcome], "inbox", { message: 1 });

    await user.click(await screen.findByRole("button", { name: "Archive" }));

    // The inbox is empty now: it explains how to send mail to it.
    expect(await screen.findByRole("heading", { name: "No messages yet" })).toBeInTheDocument();
    expect(api.messages()[0]?.folder).toBe("archive");
    expect(await screen.findByText("127.0.0.1")).toBeInTheDocument();
    expect(screen.getByText("1026")).toBeInTheDocument();
  });

  it("moves a message to trash and back from the archive", async () => {
    const user = setup([{ ...welcome, folder: "archive" }], "archive", { message: 1 });

    expect(await screen.findByRole("button", { name: "Move to inbox" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Move to trash" }));

    await waitFor(() => expect(api.messages()[0]?.folder).toBe("trash"));
    expect(
      await screen.findByRole("heading", { name: "No archived messages" }),
    ).toBeInTheDocument();
  });

  it("restores and permanently deletes messages in trash", async () => {
    const trashed = mailFixture({ id: 5, subject: "Old", folder: "trash" });
    const user = setup([{ ...welcome, folder: "trash" }, trashed], "trash", { message: 1 });

    await user.click(await screen.findByRole("button", { name: "Restore to inbox" }));
    await waitFor(() => expect(api.messages()[0]?.folder).toBe("inbox"));

    await user.click(await list().findByRole("button", { name: /Old/ }));
    await user.click(await screen.findByRole("button", { name: "Delete permanently" }));

    await waitFor(() => expect(api.messages().map((message) => message.id)).toEqual([1]));
    expect(await screen.findByRole("heading", { name: "Trash is empty" })).toBeInTheDocument();
  });

  it("empties trash only after confirmation", async () => {
    const user = setup([{ ...welcome, folder: "trash" }], "trash");
    await list().findByRole("button", { name: /Welcome/ });
    const emptyButton = await screen.findByRole("button", { name: "Empty trash" });
    await waitFor(() => expect(emptyButton).toBeEnabled());

    await user.click(emptyButton);
    expect(screen.getByRole("alertdialog")).toHaveTextContent("1 messages will be deleted");
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(api.calls("DELETE", "/api/trash")).toHaveLength(0);

    await user.click(emptyButton);
    await user.click(screen.getByRole("button", { name: "Empty trash" }));

    expect(await screen.findByRole("heading", { name: "Trash is empty" })).toBeInTheDocument();
    expect(api.calls("DELETE", "/api/trash")).toHaveLength(1);
  });

  it("searches every folder from the header and narrows to one", async () => {
    const archived = mailFixture({ id: 2, subject: "Reset archived", folder: "archive" });
    const trashed = mailFixture({ id: 3, subject: "Reset trashed", folder: "trash" });
    const user = setup([welcome, archived, trashed]);
    await list().findByRole("button", { name: /Welcome/ });

    await user.type(screen.getByRole("searchbox", { name: "Search all mail" }), "reset");

    const archivedItem = await list().findByRole("button", { name: /Reset archived/ });
    expect(archivedItem).toHaveTextContent("Archive");
    expect(list().getByRole("button", { name: /Reset trashed/ })).toHaveTextContent("Trash");
    expect(list().queryByRole("button", { name: /Welcome/ })).not.toBeInTheDocument();
    expect(badge("2 results")).toBeInTheDocument();

    await user.click(screen.getByRole("radio", { name: "Trash" }));
    await waitFor(() =>
      expect(list().queryByRole("button", { name: /Reset archived/ })).not.toBeInTheDocument(),
    );

    await user.click(screen.getByRole("searchbox", { name: "Search all mail" }));
    await user.keyboard("{Escape}");
    expect(screen.getByRole("searchbox", { name: "Search all mail" })).toHaveValue("");
    expect(await list().findByRole("button", { name: /Welcome/ })).toBeInTheDocument();
  });

  it("explains the search filters", async () => {
    const user = setup([welcome]);
    await user.click(screen.getByRole("button", { name: "Search syntax" }));
    const help = await screen.findByRole("dialog", { name: "Search syntax" });
    expect(help).toHaveTextContent("is:unread");
    expect(help).toHaveTextContent("project:shop");
  });

  it("says when a search finds nothing", async () => {
    setup([welcome], "inbox", { q: "nothing" });
    expect(await list().findByText("No messages match “nothing”.")).toBeInTheDocument();
  });

  it("groups by project and filters to one project", async () => {
    const shop = mailFixture({ id: 3, subject: "Order", project: { id: 7, name: "shop" } });
    api = createFakeMailApi([welcome, shop], [{ id: 7, name: "shop", rules: [], notify: true }]);
    vi.stubGlobal("fetch", api.fetchMock);
    const user = userEvent.setup();
    renderWithQuery(<Harness folder="inbox" />);
    await list().findByRole("button", { name: /Order/ });

    await user.click(screen.getByRole("radio", { name: "Project" }));
    expect(list().getByRole("heading", { name: "shop" })).toBeInTheDocument();
    expect(list().getByRole("heading", { name: "No project" })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Filters" }));
    await user.selectOptions(screen.getByRole("combobox", { name: "Project" }), "shop");
    await waitFor(() =>
      expect(list().queryByRole("button", { name: /Welcome/ })).not.toBeInTheDocument(),
    );

    await user.selectOptions(screen.getByRole("combobox", { name: "Project" }), "No project");
    expect(await list().findByRole("button", { name: /Welcome/ })).toBeInTheDocument();
    expect(list().queryByRole("button", { name: /Order/ })).not.toBeInTheDocument();
  });

  it("shows preview, recipient and a relative time in the list", async () => {
    setup([
      { ...welcome, receivedAt: Date.now() - 5 * 60_000, to: ["a@x.example", "b@x.example"] },
    ]);
    const item = await list().findByRole("button", { name: /Welcome/ });
    expect(item).toHaveTextContent("Hello Bob");
    expect(item).toHaveTextContent("To: a@x.example +1");
    expect(item).toHaveTextContent("5 minutes ago");
  });

  it("pages through long lists", async () => {
    const many = Array.from({ length: 30 }, (_, index) =>
      mailFixture({ id: index + 1, subject: `Mail ${index + 1}` }),
    );
    const user = setup(many, "inbox", { size: 25 });

    expect(await screen.findByText("1–25 of 30")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Previous page" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Next page" }));

    expect(await screen.findByText("26–30 of 30")).toBeInTheDocument();
    expect(list().getByRole("button", { name: /Mail 26/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Next page" })).toBeDisabled();

    await user.selectOptions(
      screen.getByRole("combobox", { name: "Messages per page" }),
      "50 per page",
    );
    expect(await screen.findByText("1–30 of 30")).toBeInTheDocument();
  });

  it("filters by when mail arrived", async () => {
    const old = mailFixture({ id: 2, subject: "Old news", receivedAt: 0 });
    const user = setup([{ ...welcome, receivedAt: Date.now() }, old]);
    await list().findByRole("button", { name: /Old news/ });

    await user.click(screen.getByRole("button", { name: "Filters" }));
    await user.selectOptions(screen.getByRole("combobox", { name: "Received" }), "Today");

    await waitFor(() =>
      expect(list().queryByRole("button", { name: /Old news/ })).not.toBeInTheDocument(),
    );
    expect(list().getByRole("button", { name: /Welcome/ })).toBeInTheDocument();
  });

  it("offers message actions on right-click", async () => {
    const user = setup([{ ...welcome, isRead: true }], "inbox", { message: 1 });
    const item = await list().findByRole("button", { name: /Welcome/ });

    fireEvent.contextMenu(item);
    await user.click(await screen.findByRole("menuitem", { name: "Mark as unread" }));
    await waitFor(() => expect(api.messages()[0]?.isRead).toBe(false));

    fireEvent.contextMenu(item);
    expect(
      await screen.findByRole("menuitem", { name: "Copy sender address" }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("menuitem", { name: "Archive" }));

    await waitFor(() => expect(api.messages()[0]?.folder).toBe("archive"));
    // The open message left the folder, so its view closes.
    expect(await screen.findByRole("heading", { name: "No messages yet" })).toBeInTheDocument();
  });

  it("downloads a message as .eml from the menu and the toolbar", async () => {
    const hrefs: string[] = [];
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (
      this: HTMLAnchorElement,
    ) {
      hrefs.push(this.href);
    });
    const user = setup([welcome], "inbox", { message: 1 });
    const item = await list().findByRole("button", { name: /Welcome/ });

    fireEvent.contextMenu(item);
    await user.click(await screen.findByRole("menuitem", { name: "Download as .eml" }));
    await user.click(await screen.findByRole("button", { name: "Download as .eml" }));

    expect(hrefs).toEqual([
      "http://localhost:3000/api/messages/1/raw?download=true",
      "http://localhost:3000/api/messages/1/raw?download=true",
    ]);
    click.mockRestore();
  });

  describe("selecting several messages", () => {
    const three = () => [
      mailFixture({ id: 1, subject: "First" }),
      mailFixture({ id: 2, subject: "Second" }),
      mailFixture({ id: 3, subject: "Third" }),
    ];
    const item = (subject: string) => list().getByRole("button", { name: new RegExp(subject) });

    it("adds with Ctrl, extends with Shift and acts on all of them", async () => {
      const user = setup(three());
      await list().findByRole("button", { name: /Third/ });

      await user.click(item("Third"));
      await user.keyboard("{Control>}");
      await user.click(item("First"));
      await user.keyboard("{/Control}");
      expect(
        await screen.findByRole("heading", { name: "2 messages selected" }),
      ).toBeInTheDocument();
      expect(item("First")).toHaveAttribute("data-checked", "true");
      expect(item("Second")).not.toHaveAttribute("data-checked");

      await user.keyboard("{Shift>}");
      await user.click(item("Second"));
      await user.keyboard("{/Shift}");
      expect(
        await screen.findByRole("heading", { name: "2 messages selected" }),
      ).toBeInTheDocument();

      await user.click(screen.getByRole("button", { name: "Move to trash" }));
      await waitFor(() =>
        expect(api.messages().filter((message) => message.folder === "trash")).toHaveLength(2),
      );
      expect(await screen.findByText("Select a message to read it.")).toBeInTheDocument();
    });

    it("selects by dragging over the list and with Ctrl+A", async () => {
      const user = setup(three());
      await list().findByRole("button", { name: /Third/ });

      fireEvent.pointerDown(item("Third"), { button: 0 });
      fireEvent.pointerEnter(item("First"), { buttons: 1 });
      expect(
        await screen.findByRole("heading", { name: "3 messages selected" }),
      ).toBeInTheDocument();

      await user.keyboard("{Escape}");
      item("Second").focus();
      await user.keyboard("{Escape}");
      expect(screen.queryByRole("heading", { name: /messages selected/ })).not.toBeInTheDocument();

      await user.keyboard("{Control>}a{/Control}");
      expect(
        await screen.findByRole("heading", { name: "3 messages selected" }),
      ).toBeInTheDocument();
    });

    it("offers the actions for all selected messages on right-click", async () => {
      const user = setup(three());
      await list().findByRole("button", { name: /Third/ });
      await user.click(item("Third"));
      await user.keyboard("{Control>}");
      await user.click(item("Second"));
      await user.keyboard("{/Control}");

      fireEvent.contextMenu(item("Second"));
      await user.click(await screen.findByRole("menuitem", { name: "Archive (2)" }));
      await waitFor(() =>
        expect(api.messages().filter((message) => message.folder === "archive")).toHaveLength(2),
      );
    });
  });

  it("opens the message menu from the keyboard", async () => {
    const user = setup([welcome]);
    const item = await list().findByRole("button", { name: /Welcome/ });
    item.focus();

    await user.keyboard("{Shift>}{F10}{/Shift}");
    fireEvent.contextMenu(item);

    expect(await screen.findByRole("menu")).toBeInTheDocument();
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("menu")).not.toBeInTheDocument());
  });

  it("deletes for good from the menu in trash", async () => {
    const trashed = mailFixture({ id: 4, subject: "Old", folder: "trash" });
    const user = setup(
      [trashed, mailFixture({ id: 5, subject: "Keep", folder: "trash" })],
      "trash",
    );
    const item = await list().findByRole("button", { name: /Old/ });

    fireEvent.contextMenu(item);
    await user.click(await screen.findByRole("menuitem", { name: "Delete permanently" }));

    await waitFor(() => expect(api.messages().map((message) => message.id)).toEqual([5]));
  });

  it("shows an error with retry when the list cannot be loaded", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 500 })),
    );
    renderWithQuery(<Harness folder="inbox" />);

    expect(await screen.findByText("Messages could not be loaded.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });
});
