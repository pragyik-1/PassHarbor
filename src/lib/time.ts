/**
 * Time formatting helpers for the vault dashboard.
 *
 * All timestamps from the backend are unix seconds (i64). Convert with
 * `new Date(seconds * 1000)` before using JS Date APIs.
 */

const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;

/**
 * Return a short human-readable relative time string, e.g. "just now",
 * "3 min ago", "2 days ago", "Jan 5, 2025".
 *
 * @param seconds Unix timestamp in seconds.
 */
export function timeAgo(seconds: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - seconds;

  if (diff < 0) return formatDate(seconds);
  if (diff < 60) return "just now";
  if (diff < HOUR) {
    const m = Math.floor(diff / MINUTE);
    return `${m} min ago`;
  }
  if (diff < DAY) {
    const h = Math.floor(diff / HOUR);
    return `${h}h ago`;
  }
  if (diff < WEEK) {
    const d = Math.floor(diff / DAY);
    return d === 1 ? "yesterday" : `${d} days ago`;
  }
  if (diff < MONTH) {
    const w = Math.floor(diff / WEEK);
    return w === 1 ? "1 week ago" : `${w} weeks ago`;
  }
  if (diff < YEAR) {
    const mo = Math.floor(diff / MONTH);
    return mo === 1 ? "1 month ago" : `${mo} months ago`;
  }
  const y = Math.floor(diff / YEAR);
  return y === 1 ? "1 year ago" : `${y} years ago`;
}

/**
 * Return a full, locale-formatted date string for the given unix timestamp.
 *
 * @param seconds Unix timestamp in seconds.
 */
export function formatDate(seconds: number): string {
  return new Date(seconds * 1000).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/**
 * Return a full date + time string, e.g. "Jan 5, 2025, 14:32".
 *
 * @param seconds Unix timestamp in seconds.
 */
export function formatDateTime(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}
