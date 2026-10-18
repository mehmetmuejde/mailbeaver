import { useForm } from "@tanstack/react-form";
import { Plus, Save, X } from "lucide-react";
import { ApiError, RULE_KINDS, type RuleKind } from "@/shared/api";
import { fieldError } from "@/shared/lib/form";
import { Button } from "@/shared/ui/button";
import { IconButton } from "@/shared/ui/icon-button";
import { SelectField } from "@/shared/ui/select-field";
import { SwitchField } from "@/shared/ui/switch-field";
import { TextField } from "@/shared/ui/text-field";
import { useSaveProject } from "../api/use-save-project";
import {
  NAME_MAX_LENGTH,
  type ProjectFormValues,
  projectFormSchema,
  RULE_KIND_LABELS,
} from "../model/project-form";
import { SettingsRuleValueField } from "./SettingsRuleValueField";

const KIND_OPTIONS = RULE_KINDS.map((kind) => ({ value: kind, label: RULE_KIND_LABELS[kind] }));
const NEW_RULE = { kind: "sender_domain", value: "" } as const;

function isRuleKind(value: string): value is RuleKind {
  return RULE_KINDS.some((kind) => kind === value);
}

type SettingsProjectFormProps = {
  /** `null` creates a new project. */
  projectId: number | null;
  initial: ProjectFormValues;
  onDone: () => void;
};

export function SettingsProjectForm({ projectId, initial, onDone }: SettingsProjectFormProps) {
  const save = useSaveProject(projectId);
  const form = useForm({
    defaultValues: initial,
    validators: { onSubmit: projectFormSchema },
    onSubmit: async ({ value }) => {
      await save.mutateAsync(projectFormSchema.parse(value));
      onDone();
    },
  });
  const serverError =
    save.error instanceof ApiError && save.error.detail !== null
      ? save.error.detail
      : "The project could not be saved.";

  return (
    <form
      noValidate
      aria-label={projectId === null ? "New project" : "Edit project"}
      className="grid gap-3 rounded-lg border p-3"
      onSubmit={(event) => {
        event.preventDefault();
        // Errors are shown at the fields and in the alert below.
        form.handleSubmit().catch(() => undefined);
      }}
    >
      <form.Field name="name">
        {(field) => (
          <TextField
            label="Name"
            value={field.state.value}
            maxLength={NAME_MAX_LENGTH}
            onChange={field.handleChange}
            onBlur={field.handleBlur}
            error={fieldError(field.state.meta.errors)}
          />
        )}
      </form.Field>
      <form.Field name="rules" mode="array">
        {(rules) => (
          <fieldset className="grid gap-2">
            <legend className="mb-1 text-sm font-medium">Rules</legend>
            <p className="text-xs text-muted-foreground">
              Mail matching any rule lands in this project. A sender or SMTP user belongs to one
              project only.
            </p>
            {rules.state.value.map((rule, index) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: TanStack Form addresses array fields by index.
              <div key={index} className="flex items-start gap-2">
                <form.Field name={`rules[${index}].kind`}>
                  {(field) => (
                    <SelectField
                      label={`Rule ${index + 1} type`}
                      hideLabel
                      options={KIND_OPTIONS}
                      value={field.state.value}
                      onChange={(value) => {
                        if (isRuleKind(value)) {
                          field.handleChange(value);
                        }
                      }}
                    />
                  )}
                </form.Field>
                <form.Field name={`rules[${index}].value`}>
                  {(field) => (
                    <SettingsRuleValueField
                      index={index}
                      kind={rule.kind}
                      value={field.state.value}
                      onChange={field.handleChange}
                      onBlur={field.handleBlur}
                      error={fieldError(field.state.meta.errors)}
                    />
                  )}
                </form.Field>
                <IconButton
                  label={`Remove rule ${index + 1}`}
                  icon={<X />}
                  onClick={() => rules.removeValue(index)}
                />
              </div>
            ))}
            <div>
              <Button size="sm" icon={<Plus />} onClick={() => rules.pushValue({ ...NEW_RULE })}>
                Add rule
              </Button>
            </div>
          </fieldset>
        )}
      </form.Field>
      <form.Field name="notify">
        {(field) => (
          <SwitchField
            label="Notify about new mail"
            description="Counts when notifications are limited to projects."
            checked={field.state.value}
            onCheckedChange={field.handleChange}
          />
        )}
      </form.Field>
      {save.isError ? <p role="alert">{serverError}</p> : null}
      <div className="flex gap-2">
        <form.Subscribe selector={(state) => state.isSubmitting}>
          {(isSubmitting) => (
            <Button type="submit" variant="default" icon={<Save />} disabled={isSubmitting}>
              {isSubmitting ? "Saving…" : "Save project"}
            </Button>
          )}
        </form.Subscribe>
        <Button variant="ghost" icon={<X />} onClick={onDone}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
