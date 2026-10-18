/** The bare address of a `From` value such as `Shop <noreply@shop.example>`. */
export function senderAddress(from: string): string {
  const match = from.match(/<([^<>]+)>\s*$/);
  return (match?.[1] ?? from).trim();
}
