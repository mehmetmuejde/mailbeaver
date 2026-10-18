import { useState } from "react";
import type { MessageDetail } from "@/shared/api";
import { type TabItem, Tabs } from "@/shared/ui/tabs";
import type { PreviewDevice } from "../model/preview-device";
import { MailHeaderTable } from "./MailHeaderTable";
import { MailHtmlPreview } from "./MailHtmlPreview";
import { MailLinkList } from "./MailLinkList";
import { MailPreviewDeviceSwitch } from "./MailPreviewDeviceSwitch";
import { MailRawSource } from "./MailRawSource";

type MailMessageBodyProps = {
  message: MessageDetail;
};

export function MailMessageBody({ message }: MailMessageBodyProps) {
  // Kept here so the choice survives a look at another tab.
  const [device, setDevice] = useState<PreviewDevice>("desktop");
  const tabs: TabItem[] = [];
  if (message.html !== null) {
    tabs.push({
      id: "html",
      label: "HTML",
      panel: <MailHtmlPreview html={message.html} device={device} />,
      actions: <MailPreviewDeviceSwitch device={device} onChange={setDevice} />,
    });
  }
  if (message.text !== null) {
    tabs.push({
      id: "text",
      label: "Text",
      panel: (
        <pre className="font-mono text-xs break-words whitespace-pre-wrap">{message.text}</pre>
      ),
    });
  }
  tabs.push(
    {
      id: "links",
      label: "Links",
      panel: <MailLinkList html={message.html} text={message.text} />,
    },
    { id: "headers", label: "Headers", panel: <MailHeaderTable headers={message.headers} /> },
    { id: "raw", label: "Source", panel: <MailRawSource messageId={message.id} /> },
  );

  return <Tabs label="Message views" tabs={tabs} appearance="buttons" />;
}
