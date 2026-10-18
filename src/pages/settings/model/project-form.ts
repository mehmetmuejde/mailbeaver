import { z } from "zod";
import { RULE_KINDS, type RuleKind } from "@/shared/api";

export const NAME_MAX_LENGTH = 64;
const RULE_VALUE_MAX_LENGTH = 254;
const MAX_RULES = 50;

export const RULE_KIND_LABELS: Record<RuleKind, string> = {
  sender_address: "Sender address",
  sender_domain: "Sender domain",
  auth_user: "SMTP user",
};

export const RULE_PLACEHOLDERS: Record<RuleKind, string> = {
  sender_address: "noreply@shop.example",
  sender_domain: "shop.example",
  auth_user: "shop",
};

const ruleSchema = z
  .object({
    kind: z.enum(RULE_KINDS),
    value: z.string().trim().min(1, { error: "Enter a value" }).max(RULE_VALUE_MAX_LENGTH),
  })
  .superRefine((rule, context) => {
    if (rule.kind === "sender_address" && !z.email().safeParse(rule.value).success) {
      context.addIssue({ code: "custom", path: ["value"], message: "Enter an email address" });
    }
    if (rule.kind === "sender_domain" && (rule.value.includes("@") || !rule.value.includes("."))) {
      context.addIssue({
        code: "custom",
        path: ["value"],
        message: "Enter a domain like shop.example",
      });
    }
  });

/** Client-side checks for convenience; the server validates again and decides. */
export const projectFormSchema = z.object({
  name: z.string().trim().min(1, { error: "Enter a name" }).max(NAME_MAX_LENGTH),
  rules: z.array(ruleSchema).max(MAX_RULES, { error: `At most ${MAX_RULES} rules` }),
  notify: z.boolean(),
});
export type ProjectFormValues = z.input<typeof projectFormSchema>;

export const EMPTY_PROJECT: ProjectFormValues = { name: "", rules: [], notify: true };
