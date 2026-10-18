/**
 * The first message among a field's errors. Schema validators report issue objects,
 * hand-written validators plain strings.
 */
export function fieldError(errors: readonly unknown[]): string | undefined {
  for (const error of errors) {
    if (typeof error === "string") {
      return error;
    }
    if (
      typeof error === "object" &&
      error !== null &&
      "message" in error &&
      typeof error.message === "string"
    ) {
      return error.message;
    }
  }
  return undefined;
}
