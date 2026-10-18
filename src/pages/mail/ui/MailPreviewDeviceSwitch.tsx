import { Monitor, Smartphone } from "lucide-react";
import { SegmentedControl, type SegmentedOption } from "@/shared/ui/segmented-control";
import type { PreviewDevice } from "../model/preview-device";

const DEVICE_OPTIONS: readonly SegmentedOption<PreviewDevice>[] = [
  { value: "desktop", label: "Desktop", icon: <Monitor /> },
  { value: "mobile", label: "Mobile", icon: <Smartphone /> },
];

type MailPreviewDeviceSwitchProps = {
  device: PreviewDevice;
  onChange: (device: PreviewDevice) => void;
};

export function MailPreviewDeviceSwitch({ device, onChange }: MailPreviewDeviceSwitchProps) {
  return (
    <SegmentedControl
      label="Preview width"
      options={DEVICE_OPTIONS}
      value={device}
      onValueChange={onChange}
    />
  );
}
