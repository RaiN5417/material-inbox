import type { DragEvent } from "react";

// Shared drag payload key so a card in the Inbox (gallery or list view) can
// be dropped onto a Group folder in the sidebar — both sides need to agree
// on the same custom MIME type.
export const FILE_DRAG_MIME = "application/x-download-inbox-file-id";

// The browser's default drag image is a full snapshot of the dragged
// element (an entire gallery card, semi-transparent) — heavy and hard to
// read while dragging. This swaps it for a small pill, the way Finder/Eagle
// show a compact label under the cursor instead.
export function setDragPreview(e: DragEvent, label: string) {
  const text = label.length > 28 ? `${label.slice(0, 27)}…` : label;
  const pill = document.createElement("div");
  pill.textContent = text;
  Object.assign(pill.style, {
    position: "fixed",
    top: "-1000px",
    left: "-1000px",
    padding: "6px 14px",
    borderRadius: "999px",
    background: "var(--accent)",
    color: "var(--accent-text)",
    fontSize: "12px",
    fontWeight: "700",
    fontFamily: "inherit",
    whiteSpace: "nowrap",
    boxShadow: "0 8px 20px rgba(0, 0, 0, 0.25)",
    pointerEvents: "none",
  });
  document.body.appendChild(pill);
  e.dataTransfer.setDragImage(pill, 16, 16);
  // The browser snapshots the element synchronously during dragstart, so
  // it's safe — and necessary — to clean it up right after.
  requestAnimationFrame(() => pill.remove());
}
