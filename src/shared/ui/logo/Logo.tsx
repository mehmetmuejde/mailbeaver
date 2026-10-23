import mascot from "./mailbeaver.png";

/**
 * Mascot and wordmark. The rounded system font keeps the app offline and inside its CSP;
 * the wordmark is plain text, so it reads as "Mailbeaver".
 */
type LogoProps = {
  /** Marks a development build next to the wordmark. */
  dev?: boolean;
};

export function Logo({ dev = false }: LogoProps) {
  return (
    <span className="inline-flex items-center gap-[0.42em] text-2xl leading-none">
      <img src={mascot} alt="" className="size-[1.35em] shrink-0 object-contain" />
      <span className="font-logo font-extrabold tracking-[-0.055em] whitespace-nowrap">
        <span className="text-slate-800 dark:text-slate-50">Mail</span>
        <span className="text-blue-500">beaver</span>
      </span>
      {dev ? (
        <span className="rounded bg-amber-400 px-1.5 py-0.5 text-xs font-bold tracking-normal text-amber-950 uppercase">
          <span className="sr-only">(</span>Dev<span className="sr-only">)</span>
        </span>
      ) : null}
    </span>
  );
}
