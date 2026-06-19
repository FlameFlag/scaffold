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

  export function markdownTable(
    table: ReadonlyArray<ReadonlyArray<string | null | undefined>>,
    options?: Options,
  ): string;
}
