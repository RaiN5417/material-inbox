import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { formatSize, type FileRecord } from "./lib/file";
import type { Group } from "./lib/group";

type CardState =
  | { mode: "single"; file: FileRecord }
  | { mode: "batch"; files: FileRecord[] }
  | null;

// The non-focus-stealing popup shown when one or more downloads finish
// (spec section 9/10). Single-file and batch share this window/component —
// event-engine on the Rust side decides which one fires (see
// apps/desktop/src-tauri/src/floating_card.rs).
export default function FloatingCard() {
  const [card, setCard] = useState<CardState>(null);
  const [groups, setGroups] = useState<Group[]>([]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    function loadGroups() {
      invoke<Group[]>("list_groups")
        .then(setGroups)
        .catch(() => setGroups([]));
    }

    const unlistenSingle = listen<FileRecord>("floating-card:show", (event) => {
      setCard({ mode: "single", file: event.payload });
      setBusy(false);
      loadGroups();
    });
    const unlistenBatch = listen<{ files: FileRecord[] }>("floating-card:show-batch", (event) => {
      setCard({ mode: "batch", files: event.payload.files });
      setBusy(false);
      loadGroups();
    });
    return () => {
      unlistenSingle.then((fn) => fn());
      unlistenBatch.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        void dismiss();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  async function dismiss() {
    await getCurrentWindow().hide();
  }

  function fileIds(): string[] {
    if (!card) return [];
    return card.mode === "single" ? [card.file.id] : card.files.map((f) => f.id);
  }

  async function markLater() {
    if (busy) return;
    setBusy(true);
    try {
      await Promise.all(fileIds().map((fileId) => invoke("mark_later", { fileId })));
    } finally {
      await dismiss();
    }
  }

  async function assignGroup(groupId: string) {
    if (busy) return;
    setBusy(true);
    try {
      await Promise.all(fileIds().map((fileId) => invoke("assign_group", { fileId, groupId })));
    } finally {
      await dismiss();
    }
  }

  async function markTemporary() {
    if (busy) return;
    setBusy(true);
    try {
      await Promise.all(fileIds().map((fileId) => invoke("mark_temporary", { fileId })));
    } finally {
      await dismiss();
    }
  }

  if (!card) return null;

  const groupButtons = groups.map((group) => (
    <button
      key={group.id}
      className="floating-card-group"
      disabled={busy}
      onClick={() => void assignGroup(group.id)}
    >
      {group.name}
    </button>
  ));

  return (
    <div className="floating-card">
      <div className="floating-card-header">
        <span>
          {card.mode === "single" ? "↓ New download" : `↓ ${card.files.length} new files`}
        </span>
        <button className="floating-card-close" onClick={() => void dismiss()} aria-label="Dismiss">
          ×
        </button>
      </div>

      {card.mode === "single" ? (
        <>
          <div className="floating-card-filename" title={card.file.current_name}>
            {card.file.current_name}
          </div>
          <div className="floating-card-meta">{formatSize(card.file.size_bytes)}</div>
        </>
      ) : (
        <ul className="floating-card-file-list">
          {card.files.map((file) => (
            <li key={file.id} title={file.current_name}>
              {file.current_name}
            </li>
          ))}
        </ul>
      )}

      <div className="floating-card-actions">
        {groupButtons}
        <button
          className="floating-card-group"
          disabled={busy}
          onClick={() => void markTemporary()}
        >
          Temporary
        </button>
        <button className="floating-card-later" disabled={busy} onClick={() => void markLater()}>
          Later
        </button>
      </div>
    </div>
  );
}
