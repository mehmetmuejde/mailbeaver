import { useQuery } from "@tanstack/react-query";
import { ListFilter, X } from "lucide-react";
import { projectQueries, UNASSIGNED_PROJECT } from "@/shared/api";
import { Button } from "@/shared/ui/button";
import { Popover } from "@/shared/ui/popover";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { type MailSearch, type MailSearchChange, PERIODS, type Period } from "../model/mail-search";

const ANY_TIME = "all";
const PERIOD_LABELS: Record<Period, string> = {
  today: "Today",
  "7d": "Last 7 days",
  "30d": "Last 30 days",
};
const PERIOD_OPTIONS: readonly SelectOption[] = [
  { value: ANY_TIME, label: "Any time" },
  ...PERIODS.map((period) => ({ value: period, label: PERIOD_LABELS[period] })),
];

// Select values are strings; "all" and "none" never collide with a numeric project id.
const ALL_PROJECTS = "all";

function isPeriod(value: string): value is Period {
  return PERIODS.some((period) => period === value);
}

function toProjectValue(project: MailSearch["project"]): string {
  return project === undefined ? ALL_PROJECTS : String(project);
}

function fromProjectValue(value: string): MailSearch["project"] {
  if (value === UNASSIGNED_PROJECT) {
    return UNASSIGNED_PROJECT;
  }
  const id = Number(value);
  return Number.isInteger(id) && id > 0 ? id : undefined;
}

type MailFiltersProps = {
  search: MailSearch;
  onSearchChange: MailSearchChange;
};

/** Project and time filters behind one button; the button counts the active ones. */
export function MailFilters({ search, onSearchChange }: MailFiltersProps) {
  const { data: overview } = useQuery(projectQueries.overview());
  const projectOptions: SelectOption[] = [
    { value: ALL_PROJECTS, label: "All projects" },
    { value: UNASSIGNED_PROJECT, label: "No project" },
    ...(overview?.projects ?? []).map((project) => ({
      value: String(project.id),
      label: project.name,
    })),
  ];
  const active = [search.project, search.period].filter((value) => value !== undefined).length;
  const change = (patch: Partial<MailSearch>) =>
    onSearchChange({ ...patch, message: undefined, page: undefined });

  return (
    <Popover
      label="Filters"
      trigger={
        <Button
          size="sm"
          icon={<ListFilter />}
          aria-label={active > 0 ? `Filters, ${active} active` : "Filters"}
        >
          {active > 0 ? active : null}
        </Button>
      }
    >
      <SelectField
        spread
        label="Project"
        options={projectOptions}
        value={toProjectValue(search.project)}
        onChange={(value) => change({ project: fromProjectValue(value) })}
      />
      <SelectField
        spread
        label="Received"
        options={PERIOD_OPTIONS}
        value={search.period ?? ANY_TIME}
        onChange={(value) => change({ period: isPeriod(value) ? value : undefined })}
      />
      {active > 0 ? (
        <div className="flex justify-end">
          <Button
            size="sm"
            variant="ghost"
            icon={<X />}
            onClick={() => change({ project: undefined, period: undefined })}
          >
            Clear filters
          </Button>
        </div>
      ) : null}
    </Popover>
  );
}
