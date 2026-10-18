import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { projectQueries, type RuleKind } from "@/shared/api";
import { pluralize } from "@/shared/lib/format";
import { TextField } from "@/shared/ui/text-field";
import { RULE_PLACEHOLDERS } from "../model/project-form";

type SettingsRuleValueFieldProps = {
  index: number;
  kind: RuleKind;
  value: string;
  onChange: (value: string) => void;
  onBlur: () => void;
  error?: string;
};

/** The value of a rule, with senders, domains or SMTP users of captured mail offered. */
export function SettingsRuleValueField({
  index,
  kind,
  value,
  onChange,
  onBlur,
  error,
}: SettingsRuleValueFieldProps) {
  const { data } = useQuery({
    ...projectQueries.ruleSuggestions(kind, value.trim()),
    // Keeps the list while the next keystroke loads.
    placeholderData: keepPreviousData,
  });
  const suggestions = (data ?? []).map((suggestion) => ({
    value: suggestion.value,
    label: pluralize(suggestion.count, "message"),
  }));
  return (
    <TextField
      label={`Rule ${index + 1} value`}
      hideLabel
      placeholder={RULE_PLACEHOLDERS[kind]}
      value={value}
      onChange={onChange}
      onBlur={onBlur}
      error={error}
      suggestions={suggestions}
    />
  );
}
