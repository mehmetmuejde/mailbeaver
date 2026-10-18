import type { ReactNode } from "react";

type EmptyStateProps = {
  icon: ReactNode;
  title: string;
  description: string;
  children?: ReactNode;
};

/** Fills an empty view with an icon, a short title and what to do next. */
export function EmptyState({ icon, title, description, children }: EmptyStateProps) {
  return (
    <div className="m-auto grid max-w-md justify-items-center gap-3 p-8 text-center">
      <span
        aria-hidden="true"
        className="flex size-14 items-center justify-center rounded-full bg-muted text-muted-foreground [&_svg]:size-7"
      >
        {icon}
      </span>
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="text-muted-foreground">{description}</p>
      {children === undefined ? null : <div className="mt-2 w-full text-left">{children}</div>}
    </div>
  );
}
