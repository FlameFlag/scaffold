import { isObject } from "../wasm/object";
import { isWasmRange, type WasmRangeLike } from "../wasm/range";
import { isNonEmptyString } from "../wasm/string";

export const semanticTokenTypeNames = [
  "function",
  "keyword",
  "string",
  "comment",
  "parameter",
] as const;

export const semanticTokenModifierNames = [
  "defaultLibrary",
  "documentation",
  "deprecated",
  "definition",
] as const;

export interface WasmSemanticToken extends WasmRangeLike {
  text: string;
  token_type: WasmSemanticTokenType;
  modifiers: WasmSemanticTokenModifier[];
}

export type WasmSemanticTokenType = (typeof semanticTokenTypeNames)[number];
export type WasmSemanticTokenModifier =
  (typeof semanticTokenModifierNames)[number];

const semanticTokenTypes = new Set<string>(semanticTokenTypeNames);
const semanticTokenModifiers = new Set<string>(semanticTokenModifierNames);

export function isWasmSemanticToken(
  value: unknown,
): value is WasmSemanticToken {
  if (!isObject(value) || !isWasmRange(value)) {
    return false;
  }
  const token = value as WasmRangeLike & Record<string, unknown>;
  return (
    isNonEmptyString(token.text) &&
    typeof token.token_type === "string" &&
    semanticTokenTypes.has(token.token_type) &&
    Array.isArray(token.modifiers) &&
    token.modifiers.every(
      (modifier): modifier is WasmSemanticTokenModifier =>
        typeof modifier === "string" && semanticTokenModifiers.has(modifier),
    )
  );
}
