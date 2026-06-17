declare module "markdown-table" {
  type Align = "l" | "r" | "c" | "";

  interface Options {
    align?: Align | Align[];
    padding?: boolean;
    delimiterStart?: boolean;
    delimiterEnd?: boolean;
    alignDelimiters?: boolean;
    stringLength?: (value: string) => number;
  }

  export default function markdownTable(
    table: string[][],
    options?: Options,
  ): string;
}
