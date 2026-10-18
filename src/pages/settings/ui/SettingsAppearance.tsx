import {
  setThemePreference,
  THEME_PREFERENCES,
  type ThemePreference,
  useThemePreference,
} from "@/shared/lib/theme";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";

const THEME_LABELS: Record<ThemePreference, string> = {
  system: "Same as the system",
  light: "Light",
  dark: "Dark",
};

const OPTIONS: readonly SelectOption[] = THEME_PREFERENCES.map((theme) => ({
  value: theme,
  label: THEME_LABELS[theme],
}));

function isTheme(value: string): value is ThemePreference {
  return THEME_PREFERENCES.some((theme) => theme === value);
}

/** Applies at once and is remembered on this device. */
export function SettingsAppearance() {
  const theme = useThemePreference();
  return (
    <section aria-labelledby="appearance-heading" className="grid gap-3">
      <h2 id="appearance-heading" className="text-base font-semibold">
        Appearance
      </h2>
      <SelectField
        label="Theme"
        options={OPTIONS}
        value={theme}
        onChange={(value) => {
          if (isTheme(value)) {
            setThemePreference(value);
          }
        }}
      />
    </section>
  );
}
