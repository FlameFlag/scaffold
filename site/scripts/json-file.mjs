import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";

export async function readJsonFile(path, label = path) {
  let text;

  try {
    text = await readFile(path, "utf8");
  } catch (error) {
    throw new Error(`Could not read ${label}: ${errorMessage(error)}`);
  }

  return parseJsonText(text, label);
}

export function readJsonFileSync(path, label = path) {
  let text;

  try {
    text = readFileSync(path, "utf8");
  } catch (error) {
    throw new Error(`Could not read ${label}: ${errorMessage(error)}`);
  }

  return parseJsonText(text, label);
}

export function parseJsonText(text, label) {
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`Could not parse ${label} as JSON: ${errorMessage(error)}`);
  }
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
