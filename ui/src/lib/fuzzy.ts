/**
 * Lightweight fuzzy matching for the results search box.
 *
 * The old search was a strict substring match, so "drvdata" wouldn't find "DerivedData" and
 * "nmod" wouldn't find "node_modules". This adds order-preserving subsequence matching (every
 * query char appears in order) on top of substring, which is forgiving without pulling in a
 * dependency. Matching is case-insensitive and ignores separators in the query.
 */

/** True if every character of `query` appears in `text` in order (case-insensitive). */
export function fuzzyMatch(query: string, text: string): boolean {
  if (!query) return true;
  const q = query.toLowerCase().replace(/[\s/_\-.]/g, '');
  const t = text.toLowerCase();
  if (!q) return true;
  // Fast path: direct substring.
  if (t.includes(q)) return true;
  // Subsequence match.
  let qi = 0;
  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) qi++;
  }
  return qi === q.length;
}

/** True if `query` fuzzy-matches ANY of the provided fields. */
export function fuzzyMatchAny(query: string, fields: Array<string | undefined>): boolean {
  if (!query) return true;
  return fields.some((f) => (f ? fuzzyMatch(query, f) : false));
}
