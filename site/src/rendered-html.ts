export function assertSafeRenderedHtml(html: string, label: string): void {
  if (containsUnsafeRenderedHtml(html)) {
    throw new Error(`unsafe rendered HTML in ${label}`);
  }
}

export function containsUnsafeRenderedHtml(html: string): boolean {
  return (
    /<\/?(?:script|iframe|object|embed|style|link|meta|svg|math|video|audio|source|track|form|input|button|textarea|select|option)\b/i.test(
      html,
    ) ||
    /\son[a-z]+\s*=/i.test(html) ||
    /javascript:/i.test(html)
  );
}
