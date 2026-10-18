import type { HeaderLine } from "@/shared/api";

type MailHeaderTableProps = {
  headers: readonly HeaderLine[];
};

export function MailHeaderTable({ headers }: MailHeaderTableProps) {
  return (
    <table className="w-full border-collapse text-xs">
      <caption className="sr-only">Message headers</caption>
      <thead>
        <tr>
          <th scope="col" className="border-b px-2 py-1 text-left">
            Name
          </th>
          <th scope="col" className="border-b px-2 py-1 text-left">
            Value
          </th>
        </tr>
      </thead>
      <tbody>
        {headers.map((header, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: header names repeat and the list never reorders.
          <tr key={`${index}-${header.name}`}>
            <th scope="row" className="border-b px-2 py-1 text-left align-top font-medium">
              {header.name}
            </th>
            <td className="border-b px-2 py-1 font-mono break-all">{header.value}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
