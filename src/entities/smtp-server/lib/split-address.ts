/** Splits `127.0.0.1:1026` or `[::1]:1026` into host and port. */
export function splitAddress(address: string): { host: string; port: string } {
  const separator = address.lastIndexOf(":");
  if (separator === -1) {
    return { host: address, port: "" };
  }
  return {
    host: address.slice(0, separator).replace(/^\[(.*)\]$/, "$1"),
    port: address.slice(separator + 1),
  };
}
