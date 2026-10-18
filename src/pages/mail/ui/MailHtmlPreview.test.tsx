import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import type { PreviewDevice } from "../model/preview-device";
import { MailHtmlPreview } from "./MailHtmlPreview";
import { MailPreviewDeviceSwitch } from "./MailPreviewDeviceSwitch";

function Preview() {
  const [device, setDevice] = useState<PreviewDevice>("desktop");
  return (
    <>
      <MailPreviewDeviceSwitch device={device} onChange={setDevice} />
      <MailHtmlPreview html="<p>Hi</p>" device={device} />
    </>
  );
}

describe("MailHtmlPreview", () => {
  it("renders the HTML sandboxed and switches to a phone width", async () => {
    const user = userEvent.setup();
    render(<Preview />);

    const frame = screen.getByTitle("HTML preview");
    expect(frame).toHaveAttribute("sandbox", "");
    expect(frame).toHaveAttribute("srcdoc", "<p>Hi</p>");
    expect(frame).toHaveClass("w-full");

    await user.click(screen.getByRole("radio", { name: "Mobile" }));

    expect(screen.getByRole("radio", { name: "Mobile" })).toBeChecked();
    expect(frame).toHaveClass("w-[390px]");
  });
});
