import { Link } from "@tanstack/react-router";

export function NotFound() {
  return (
    <div className="mx-auto grid max-w-xl gap-3 p-6">
      <h1 className="text-lg font-semibold">Page not found</h1>
      <p>
        <Link to="/inbox" className="text-primary underline underline-offset-2">
          Go to the inbox
        </Link>
      </p>
    </div>
  );
}
