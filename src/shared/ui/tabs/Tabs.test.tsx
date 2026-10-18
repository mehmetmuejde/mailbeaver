import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { Tabs } from "./Tabs";

const tabs = [
  { id: "a", label: "Alpha", panel: <p>Alpha panel</p> },
  { id: "b", label: "Beta", panel: <p>Beta panel</p> },
  { id: "c", label: "Gamma", panel: <p>Gamma panel</p> },
];

describe("Tabs", () => {
  it("shows the default tab and switches on click", async () => {
    const user = userEvent.setup();
    render(<Tabs label="Views" tabs={tabs} defaultTabId="b" />);

    expect(screen.getByRole("tab", { name: "Beta" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel", { name: "Beta" })).toHaveTextContent("Beta panel");

    await user.click(screen.getByRole("tab", { name: "Gamma" }));
    expect(screen.getByRole("tabpanel", { name: "Gamma" })).toHaveTextContent("Gamma panel");
    expect(screen.queryByText("Beta panel")).not.toBeInTheDocument();
  });

  it("moves selection and focus with the keyboard", async () => {
    const user = userEvent.setup();
    render(<Tabs label="Views" tabs={tabs} />);

    await user.tab();
    expect(screen.getByRole("tab", { name: "Alpha" })).toHaveFocus();

    await user.keyboard("{ArrowLeft}");
    expect(screen.getByRole("tab", { name: "Gamma" })).toHaveFocus();
    expect(screen.getByRole("tab", { name: "Gamma" })).toHaveAttribute("aria-selected", "true");

    await user.keyboard("{Home}");
    expect(screen.getByRole("tab", { name: "Alpha" })).toHaveFocus();

    await user.keyboard("{End}");
    expect(screen.getByRole("tab", { name: "Gamma" })).toHaveFocus();

    await user.keyboard("{ArrowRight}");
    expect(screen.getByRole("tab", { name: "Alpha" })).toHaveFocus();
  });

  it("shows the actions of the active tab next to the tab row", async () => {
    const user = userEvent.setup();
    render(
      <Tabs
        label="Views"
        appearance="buttons"
        tabs={[
          {
            id: "a",
            label: "Alpha",
            panel: <p>Alpha panel</p>,
            actions: <button type="button">Zoom</button>,
          },
          { id: "b", label: "Beta", panel: <p>Beta panel</p> },
        ]}
      />,
    );

    expect(screen.getByRole("button", { name: "Zoom" })).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Beta" }));
    expect(screen.queryByRole("button", { name: "Zoom" })).not.toBeInTheDocument();
    expect(screen.getByRole("tabpanel")).toHaveTextContent("Beta panel");
  });
});
