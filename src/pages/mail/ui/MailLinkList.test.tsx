import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { renderWithQuery } from "@/shared/lib/test";
import { MailLinkList } from "./MailLinkList";

const HTML = '<a href="https://shop.example/reset">Reset password</a>';

describe("MailLinkList", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("lists links and copies one", async () => {
    const user = userEvent.setup();
    const writeText = vi.spyOn(navigator.clipboard, "writeText").mockResolvedValue(undefined);
    renderWithQuery(<MailLinkList html={HTML} text={null} />);

    expect(screen.getByText("Reset password")).toBeInTheDocument();
    const copy = screen.getByRole("button", { name: "Copy link" });
    expect(copy).toHaveAccessibleDescription("https://shop.example/reset");

    await user.click(copy);

    expect(writeText).toHaveBeenCalledWith("https://shop.example/reset");
    expect(await screen.findByText("Link copied.")).toBeInTheDocument();
  });

  it("reports a failed copy", async () => {
    const user = userEvent.setup();
    vi.spyOn(navigator.clipboard, "writeText").mockRejectedValue(new Error("denied"));
    renderWithQuery(<MailLinkList html={HTML} text={null} />);

    await user.click(screen.getByRole("button", { name: "Copy link" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("The link could not be copied.");
  });

  it("checks the web links and shows their status", async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn(async () =>
      Response.json({
        results: [
          { url: "https://shop.example/reset", status: 200, ok: true, error: null },
          { url: "https://shop.example/gone", status: 404, ok: false, error: null },
          { url: "https://down.example/", status: null, ok: false, error: "timed out" },
        ],
      }),
    );
    vi.stubGlobal("fetch", fetchMock);
    renderWithQuery(
      <MailLinkList
        html={`${HTML}<a href="mailto:help@shop.example">Help</a>`}
        text="See https://shop.example/gone and https://down.example/"
      />,
    );

    await user.click(screen.getByRole("button", { name: "Check links" }));

    expect(await screen.findByText("200 OK")).toBeInTheDocument();
    expect(screen.getByText("404")).toBeInTheDocument();
    expect(screen.getByText("timed out")).toBeInTheDocument();
    const [, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(JSON.parse(String(init.body))).toEqual({
      urls: ["https://shop.example/reset", "https://shop.example/gone", "https://down.example/"],
    });
    vi.unstubAllGlobals();
  });

  it("reports when the check fails", async () => {
    const user = userEvent.setup();
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 500 })),
    );
    renderWithQuery(<MailLinkList html={HTML} text={null} />);

    await user.click(screen.getByRole("button", { name: "Check links" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("The links could not be checked.");
    vi.unstubAllGlobals();
  });

  it("says when there are no links", () => {
    renderWithQuery(<MailLinkList html={null} text="No links here" />);
    expect(screen.getByText("This message contains no links.")).toBeInTheDocument();
  });
});
