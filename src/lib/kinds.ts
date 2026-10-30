import { toast } from "@hermitk/bluenite";

export const DEFAULT_KINDS = [
  "Login",
  "Note",
  "Card",
  "Other",
] as const;

export interface KindCount {
  kind: string;
  count: number;
}

const CUSTOM_KINDS_KEY = "passharbor.custom-kinds";

function kindKey(kind: string): string {
  return kind.trim().toLowerCase();
}

export function isDefaultKind(kind: string): boolean {
  return DEFAULT_KINDS.some((defaultKind) => kindKey(defaultKind) === kindKey(kind));
}

/** Merge kind lists case-insensitively, keeping the first spelling of each kind. */
function mergeKinds(...groups: Iterable<string>[]): string[] {
  const kinds = new Map<string, string>();
  for (const group of groups) {
    for (const rawKind of group) {
      const kind = rawKind.trim();
      if (!kind) continue;
      const key = kindKey(kind);
      if (!kinds.has(key)) kinds.set(key, kind);
    }
  }
  return Array.from(kinds.values()).sort((a, b) =>
    a.localeCompare(b, undefined, { sensitivity: "base" }),
  );
}

/** Retrieve the user's custom kinds from localStorage. */
export function getCustomKinds(): string[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(CUSTOM_KINDS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (item): item is string =>
          typeof item === "string" && item.trim() !== "",
      );
    }
  } catch (e) {
    toast.show({ message: "Error", description: "An error occured while loading custom kinds, check console for more info" })
    console.log("Error loading custom kinds", e)
  }
  return [];
}

function saveCustomKinds(kinds: string[]): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(CUSTOM_KINDS_KEY, JSON.stringify(kinds));
  } catch (e) {
    toast.show({ message: "Error", description: "An error occured while saving custom kinds, check console for more info" })
    console.log("Error saving custom kinds", e)
  }
}

/**
 * Add a custom kind. Validates the name and checks for duplicates against
 * built-in and existing custom kinds, case-insensitively.
 */
export function addCustomKind(
  rawKind: string,
): { success: boolean; error?: string } {
  const kind = rawKind.trim();
  const key = kindKey(kind);
  if (!kind) {
    return { success: false, error: "Kind name cannot be empty" };
  }
  if (kind.length > 64) {
    return { success: false, error: "Kind name cannot exceed 64 characters" };
  }
  if (DEFAULT_KINDS.some((defaultKind) => kindKey(defaultKind) === key)) {
    return { success: false, error: `"${kind}" is already a built-in kind` };
  }

  const existing = getCustomKinds();
  if (existing.some((existingKind) => kindKey(existingKind) === key)) {
    return { success: false, error: `"${kind}" already exists` };
  }

  saveCustomKinds([...existing, kind]);
  return { success: true };
}

/** Remove a custom kind, case-insensitively. */
export function removeCustomKind(kind: string): void {
  const key = kindKey(kind);
  saveCustomKinds(
    getCustomKinds().filter((existingKind) => kindKey(existingKind) !== key),
  );
}

/** Remove all stored custom kinds. */
export function resetCustomKinds(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.removeItem(CUSTOM_KINDS_KEY);
  } catch {
    // Ignore storage failures.
  }
}

/** Saved custom kinds plus custom kinds used by entries, deduplicated. */
export function mergeCustomKinds(
  saved: Iterable<string>,
  entryKinds: Iterable<string> = [],
): string[] {
  return mergeKinds(
    Array.from(saved).filter((kind) => !isDefaultKind(kind)),
    Array.from(entryKinds).filter((kind) => !isDefaultKind(kind)),
  );
}

/** All built-in, custom, and entry-derived kinds, deduplicated. */
export function getAllKinds(entryKinds: Iterable<string> = []): string[] {
  return mergeKinds(DEFAULT_KINDS, getCustomKinds(), entryKinds);
}

/** Count entries per kind, treating kind names case-insensitively. */
export function countKinds(entryKinds: Iterable<string>): KindCount[] {
  const counts = new Map<string, KindCount>();
  for (const rawKind of entryKinds) {
    const kind = rawKind.trim();
    if (!kind) continue;
    const key = kindKey(kind);
    const existing = counts.get(key);
    if (existing) {
      existing.count += 1;
    } else {
      counts.set(key, { kind, count: 1 });
    }
  }
  return Array.from(counts.values()).sort((a, b) =>
    a.kind.localeCompare(b.kind, undefined, { sensitivity: "base" }),
  );
}

/** Whether an entry kind belongs to the active kind filter. */
export function kindMatches(kind: string, filter: string | null): boolean {
  return filter === null || kindKey(kind) === kindKey(filter);
}

/** Whether a specific kind is currently active/selected as the filter (case-insensitive). */
export function isKindActive(kind: string, filter: string | null): boolean {
  return filter !== null && kindKey(kind) === kindKey(filter);
}

