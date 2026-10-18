import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createFakeMailApi, mailFixture, renderWithQuery } from "@/shared/lib/test";
import { SettingsProjects } from "./SettingsProjects";

const SHOP = {
  id: 1,
  name: "Shop",
  rules: [{ kind: "sender_domain" as const, value: "shop.example" }],
  notify: true,
};

function setup() {
  const api = createFakeMailApi(
    [mailFixture({ id: 1, project: { id: 1, name: "Shop" } }), mailFixture({ id: 2 })],
    [SHOP],
  );
  vi.stubGlobal("fetch", api.fetchMock);
  const user = userEvent.setup();
  renderWithQuery(<SettingsProjects />);
  return { api, user };
}

describe("SettingsProjects", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("suggests rule values from captured mail", async () => {
    const { user } = setup();
    await user.click(await screen.findByRole("button", { name: "New project" }));
    const form = screen.getByRole("form", { name: "New project" });
    await user.click(within(form).getByRole("button", { name: "Add rule" }));
    const field = within(form).getByRole("combobox", { name: "Rule 1 value" });

    await user.type(field, "ex");
    const listId = field.getAttribute("list") ?? "";
    await waitFor(() =>
      expect(document.getElementById(listId)?.querySelector("option")).toHaveAttribute(
        "value",
        "example.com",
      ),
    );
    expect(document.getElementById(listId)?.querySelector("option")).toHaveAttribute(
      "label",
      "2 messages",
    );
  });

  it("lists projects with their rules and counts", async () => {
    setup();
    const shop = await screen.findByRole("article", { name: "Shop" });
    expect(shop).toHaveTextContent("1 message, 1 unread");
    expect(shop).toHaveTextContent("Sender domain: shop.example");
    expect(screen.getByText("1 message without a project.")).toBeInTheDocument();
  });

  it("creates a project with rules after validating them", async () => {
    const { api, user } = setup();
    await screen.findByRole("article", { name: "Shop" });

    await user.click(screen.getByRole("button", { name: "New project" }));
    const form = screen.getByRole("form", { name: "New project" });
    await user.click(within(form).getByRole("button", { name: "Add rule" }));
    await user.click(within(form).getByRole("button", { name: "Save project" }));

    expect(await within(form).findByText("Enter a name")).toBeInTheDocument();
    expect(within(form).getByText("Enter a value")).toBeInTheDocument();
    expect(api.calls("POST", "/api/projects")).toHaveLength(0);

    await user.type(within(form).getByRole("textbox", { name: "Name" }), "Blog");
    await user.selectOptions(
      within(form).getByRole("combobox", { name: "Rule 1 type" }),
      "Sender address",
    );
    await user.type(within(form).getByRole("combobox", { name: "Rule 1 value" }), "not-an-address");
    await user.click(within(form).getByRole("button", { name: "Save project" }));
    expect(await within(form).findByText("Enter an email address")).toBeInTheDocument();

    await user.clear(within(form).getByRole("combobox", { name: "Rule 1 value" }));
    await user.type(
      within(form).getByRole("combobox", { name: "Rule 1 value" }),
      " news@blog.example ",
    );
    await user.click(within(form).getByRole("switch", { name: "Notify about new mail" }));
    await user.click(within(form).getByRole("button", { name: "Save project" }));

    expect(await screen.findByRole("article", { name: "Blog" })).toBeInTheDocument();
    expect(api.projects()[1]).toEqual({
      id: 2,
      name: "Blog",
      rules: [{ kind: "sender_address", value: "news@blog.example" }],
      notify: false,
    });
  });

  it("shows the server's reason when saving fails", async () => {
    const { user } = setup();
    await screen.findByRole("article", { name: "Shop" });

    await user.click(screen.getByRole("button", { name: "New project" }));
    const form = screen.getByRole("form", { name: "New project" });
    await user.type(within(form).getByRole("textbox", { name: "Name" }), "shop");
    await user.click(within(form).getByRole("button", { name: "Save project" }));

    expect(await within(form).findByRole("alert")).toHaveTextContent(
      'a project named "shop" already exists',
    );
  });

  it("edits a project and removes a rule", async () => {
    const { api, user } = setup();
    const shop = await screen.findByRole("article", { name: "Shop" });

    await user.click(within(shop).getByRole("button", { name: "Edit" }));
    const form = screen.getByRole("form", { name: "Edit project" });
    expect(within(form).getByRole("combobox", { name: "Rule 1 value" })).toHaveValue(
      "shop.example",
    );
    await user.click(within(form).getByRole("button", { name: "Remove rule 1" }));
    await user.click(within(form).getByRole("button", { name: "Save project" }));

    await waitFor(() => expect(api.projects()[0]?.rules).toEqual([]));
    expect(await screen.findByText("No rules: assign mail by adding one.")).toBeInTheDocument();
  });

  it("cancels editing without saving", async () => {
    const { api, user } = setup();
    const shop = await screen.findByRole("article", { name: "Shop" });

    await user.click(within(shop).getByRole("button", { name: "Edit" }));
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    expect(screen.queryByRole("form")).not.toBeInTheDocument();
    expect(api.calls("PUT", "/api/projects/1")).toHaveLength(0);
  });

  it("deletes a project after confirmation", async () => {
    const { api, user } = setup();
    const shop = await screen.findByRole("article", { name: "Shop" });

    await user.click(within(shop).getByRole("button", { name: "Delete" }));
    await user.click(screen.getByRole("button", { name: "Delete project" }));

    await waitFor(() => expect(api.projects()).toEqual([]));
    expect(await screen.findByText("No projects yet.")).toBeInTheDocument();
  });
});
