import { queryOptions } from "@tanstack/react-query";
import { z } from "zod";
import { request, requestJson, sendJson } from "./client";
import { countsSchema, messageKeys } from "./messages";

export const RULE_KINDS = ["sender_address", "sender_domain", "auth_user"] as const;
const ruleKindSchema = z.enum(RULE_KINDS);
export type RuleKind = z.infer<typeof ruleKindSchema>;

const ruleSchema = z.object({ kind: ruleKindSchema, value: z.string() });

const projectSchema = z.object({
  id: z.number().int(),
  name: z.string(),
  rules: z.array(ruleSchema),
  /** Raises notifications when they are limited to projects. */
  notify: z.boolean(),
});
export type Project = z.infer<typeof projectSchema>;

const projectOverviewSchema = z.object({
  projects: z.array(projectSchema.extend(countsSchema.shape)),
  /** Messages outside Trash that no rule assigns to a project. */
  unassigned: countsSchema,
});
export type ProjectOverview = z.infer<typeof projectOverviewSchema>;

export type ProjectInput = {
  name: string;
  rules: z.infer<typeof ruleSchema>[];
  notify: boolean;
};

const ruleSuggestionSchema = z.object({ value: z.string(), count: z.number().int() });
type RuleSuggestion = z.infer<typeof ruleSuggestionSchema>;

function fetchRuleSuggestions(kind: RuleKind, prefix: string): Promise<RuleSuggestion[]> {
  const params = new URLSearchParams({ kind, q: prefix });
  return requestJson(`/api/projects/rule-suggestions?${params}`, z.array(ruleSuggestionSchema));
}

function fetchProjects(): Promise<ProjectOverview> {
  return requestJson("/api/projects", projectOverviewSchema);
}

export async function createProject(input: ProjectInput): Promise<Project> {
  const response = await sendJson("/api/projects", "POST", input);
  return projectSchema.parse(await response.json());
}

export async function updateProject({
  id,
  input,
}: {
  id: number;
  input: ProjectInput;
}): Promise<Project> {
  const response = await sendJson(`/api/projects/${id}`, "PUT", input);
  return projectSchema.parse(await response.json());
}

export async function deleteProject(id: number): Promise<void> {
  await request(`/api/projects/${id}`, { method: "DELETE" });
}

// Project counts change with every message, so they live under the message keys.
export const projectKeys = {
  all: () => [...messageKeys.all, "projects"] as const,
};

export const projectQueries = {
  overview: () => queryOptions({ queryKey: projectKeys.all(), queryFn: fetchProjects }),
  ruleSuggestions: (kind: RuleKind, prefix: string) =>
    queryOptions({
      queryKey: [...projectKeys.all(), "suggestions", kind, prefix] as const,
      queryFn: () => fetchRuleSuggestions(kind, prefix),
    }),
};
