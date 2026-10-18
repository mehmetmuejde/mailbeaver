import { ChevronLeft, ChevronRight } from "lucide-react";
import { IconButton } from "@/shared/ui/icon-button";
import { SelectField, type SelectOption } from "@/shared/ui/select-field";
import { PAGE_SIZES, type PageSize } from "../model/mail-search";

const SIZE_OPTIONS: readonly SelectOption[] = PAGE_SIZES.map((size) => ({
  value: String(size),
  label: `${size} per page`,
}));

function isPageSize(value: number): value is PageSize {
  return PAGE_SIZES.some((size) => size === value);
}

type MailPaginationProps = {
  total: number;
  /** 1-based. */
  page: number;
  size: PageSize;
  onChange: (page: number, size: PageSize) => void;
};

export function MailPagination({ total, page, size, onChange }: MailPaginationProps) {
  const pages = Math.max(1, Math.ceil(total / size));
  const first = total === 0 ? 0 : (page - 1) * size + 1;
  const last = Math.min(total, page * size);

  return (
    <nav
      aria-label="Pages"
      className="flex items-center justify-between gap-2 border-t px-3 py-2 text-xs"
    >
      <p className="text-muted-foreground">
        {first}–{last} of {total}
      </p>
      <div className="flex items-center gap-1">
        <SelectField
          label="Messages per page"
          hideLabel
          options={SIZE_OPTIONS}
          value={String(size)}
          onChange={(value) => {
            const next = Number(value);
            if (isPageSize(next)) {
              onChange(1, next);
            }
          }}
        />
        <IconButton
          label="Previous page"
          icon={<ChevronLeft />}
          disabled={page <= 1}
          onClick={() => onChange(page - 1, size)}
        />
        <IconButton
          label="Next page"
          icon={<ChevronRight />}
          disabled={page >= pages}
          onClick={() => onChange(page + 1, size)}
        />
      </div>
    </nav>
  );
}
