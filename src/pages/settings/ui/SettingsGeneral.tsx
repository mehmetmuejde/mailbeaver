import { SettingsAppearance } from "./SettingsAppearance";
import { SettingsCapture } from "./SettingsCapture";
import { SettingsServerInfo } from "./SettingsServerInfo";
import { SettingsServerLimits } from "./SettingsServerLimits";
import { SettingsUpdates } from "./SettingsUpdates";

export function SettingsGeneral() {
  return (
    <div className="grid gap-8">
      <SettingsServerInfo />
      <SettingsServerLimits />
      <SettingsCapture />
      <SettingsAppearance />
      <SettingsUpdates />
    </div>
  );
}
