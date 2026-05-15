import type { ReactNode } from "react";

const LINK_PATTERN = /((?:https?:\/\/|www\.)[^\s<>()]+|mailto:[^\s<>()]+)/gi;

export function formatFileSize(size?: number | null) {
  if (!size) return "";
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

export function renderEmailBody(body: string) {
  const parts: ReactNode[] = [];
  let lastIndex = 0;

  for (const match of body.matchAll(LINK_PATTERN)) {
    const raw = match[0];
    const index = match.index ?? 0;

    if (index > lastIndex) {
      parts.push(body.slice(lastIndex, index));
    }

    const href = raw.startsWith("www.") ? `https://${raw}` : raw;
    const trailing = href.match(/[.,;:!?)]$/)?.[0] ?? "";
    const cleanHref = trailing ? href.slice(0, -1) : href;
    const cleanLabel = trailing ? raw.slice(0, -1) : raw;

    parts.push(
      <a
        key={`${cleanHref}-${index}`}
        href={cleanHref}
        target="_blank"
        rel="noreferrer"
      >
        {cleanLabel}
      </a>
    );

    if (trailing) {
      parts.push(trailing);
    }

    lastIndex = index + raw.length;
  }

  if (lastIndex < body.length) {
    parts.push(body.slice(lastIndex));
  }

  return parts;
}
