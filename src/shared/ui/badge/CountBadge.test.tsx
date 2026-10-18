import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CountBadge } from "./CountBadge";

describe("CountBadge", () => {
  it("reads the count with its meaning and hides zero", () => {
    const { rerender, container } = render(<CountBadge count={3} label="unread" />);
    expect(screen.getByText("3")).toHaveTextContent("3 unread");

    rerender(<CountBadge count={0} label="unread" />);
    expect(container).toBeEmptyDOMElement();
  });
});
