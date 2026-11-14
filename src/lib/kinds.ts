export const DEFAULT_KINDS = ['Login', 'Note', 'Card', 'Other'] as const

export function kindKey(kind: string): string {
  return kind.trim().toLowerCase()
}

export function isDefaultKind(kind: string): boolean {
  return DEFAULT_KINDS.some((defaultKind) => kindKey(defaultKind) === kindKey(kind))
}

/** Merge kind lists case-insensitively, keeping the first spelling of each kind. */
export function mergeKinds(...groups: Iterable<string>[]): string[] {
  const kinds = new Map<string, string>()
  for (const group of groups) {
    for (const rawKind of group) {
      const kind = rawKind.trim()
      if (!kind) continue
      const key = kindKey(kind)
      if (!kinds.has(key)) kinds.set(key, kind)
    }
  }
  return Array.from(kinds.values()).sort((a, b) =>
    a.localeCompare(b, undefined, { sensitivity: 'base' }),
  )
}

/** Whether a specific kind is currently active/selected as the filter (case-insensitive). */
export function isKindActive(kind: string, filter: string | null): boolean {
  return filter !== null && kindKey(kind) === kindKey(filter)
}
