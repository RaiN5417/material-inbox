import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { formatRelative, formatSize, type FileRecord } from "./lib/file";
import type { Group } from "./lib/group";
import {
  GroupsIcon,
  HistoryIcon,
  InboxIcon,
  SettingsIcon,
  TemporaryIcon,
} from "./icons";

type Tab = "inbox" | "groups" | "temporary";

// A file plus, once known, the id of the move operation that organized it —
// that's what Undo needs. No dedicated History view yet (later milestone),
// so this list doubles as a lightweight one.
interface TrackedFile extends FileRecord {
  operationId?: string;
}

interface OrganizedEvent {
  file: FileRecord;
  operation_id: string;
}

// Main window shell. Full Inbox (pending/later/failed) and History/Settings
// are later milestones — see docs/architecture.md. Those nav entries are
// shown disabled so the app reads as a real sidebar app instead of getting
// rebuilt from scratch each milestone.
export default function App() {
  const [tab, setTab] = useState<Tab>("inbox");
  const [coreStatus, setCoreStatus] = useState<"checking" | "ok" | "error">("checking");
  const [readyFiles, setReadyFiles] = useState<TrackedFile[]>([]);

  useEffect(() => {
    invoke<string>("ping")
      .then(() => setCoreStatus("ok"))
      .catch(() => setCoreStatus("error"));
  }, []);

  useEffect(() => {
    const unlistenReady = listen<FileRecord>("file-ready", (event) => {
      setReadyFiles((prev) => [event.payload, ...prev].slice(0, 50));
    });
    const unlistenOrganized = listen<OrganizedEvent>("file-organized", (event) => {
      const { file, operation_id: operationId } = event.payload;
      setReadyFiles((prev) => {
        const tracked: TrackedFile = { ...file, operationId };
        const exists = prev.some((f) => f.id === file.id);
        if (exists) {
          return prev.map((f) => (f.id === file.id ? tracked : f));
        }
        return [tracked, ...prev].slice(0, 50);
      });
    });
    return () => {
      unlistenReady.then((fn) => fn());
      unlistenOrganized.then((fn) => fn());
    };
  }, []);

  async function undoOperation(operationId: string) {
    try {
      const restored = await invoke<FileRecord>("undo_operation", { operationId });
      setReadyFiles((prev) =>
        prev.map((f) => (f.id === restored.id ? { ...restored, operationId: undefined } : f)),
      );
    } catch (err) {
      window.alert(`Couldn't undo: ${String(err)}`);
    }
  }

  return (
    <div className="shell">
      <nav className="sidebar">
        <div className="brand">
          <img src="/icon.png" alt="" className="brand-icon" />
          <span>Download Inbox</span>
        </div>

        <button
          className={`nav-item ${tab === "inbox" ? "active" : ""}`}
          onClick={() => setTab("inbox")}
        >
          <InboxIcon /> Inbox
          {readyFiles.length > 0 && <span className="nav-badge">{readyFiles.length}</span>}
        </button>
        <button
          className={`nav-item ${tab === "groups" ? "active" : ""}`}
          onClick={() => setTab("groups")}
        >
          <GroupsIcon /> Groups
        </button>
        <button
          className={`nav-item ${tab === "temporary" ? "active" : ""}`}
          onClick={() => setTab("temporary")}
        >
          <TemporaryIcon /> Temporary
        </button>
        <button className="nav-item" disabled title="Coming in a later milestone">
          <HistoryIcon /> History
        </button>
        <button className="nav-item" disabled title="Coming in a later milestone">
          <SettingsIcon /> Settings
        </button>

        <div className="core-status">
          <span className={`status-dot status-${coreStatus}`} />
          Rust core {coreStatus === "checking" ? "checking…" : coreStatus}
        </div>
      </nav>

      <main className="content">
        {tab === "inbox" && (
          <InboxPanel files={readyFiles} onUndo={(id) => void undoOperation(id)} />
        )}
        {tab === "groups" && <GroupsPanel />}
        {tab === "temporary" && <TemporaryPanel />}
      </main>
    </div>
  );
}

function InboxPanel({
  files,
  onUndo,
}: {
  files: TrackedFile[];
  onUndo: (operationId: string) => void;
}) {
  return (
    <>
      <header className="panel-header">
        <h1>Inbox</h1>
        <p>
          Tell files where they belong while you still remember. Files also pop up as a Floating
          Card the moment they finish downloading — this list is a running log of the same thing.
        </p>
      </header>

      {files.length === 0 ? (
        <EmptyState
          icon={<InboxIcon width={28} height={28} />}
          text="Drop a file into your Downloads folder to see it show up here."
        />
      ) : (
        <ul className="file-list">
          {files.map((file) => (
            <li key={file.id} className="file-row">
              <span className="file-name" title={file.current_name}>
                {file.current_name}
              </span>
              <span className="file-size">{formatSize(file.size_bytes)}</span>
              <span className={`status-badge status-badge-${file.status}`}>{file.status}</span>
              {file.status === "organized" && file.operationId && (
                <button className="btn-link" onClick={() => onUndo(file.operationId!)}>
                  Undo
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </>
  );
}

function GroupsPanel() {
  const [groups, setGroups] = useState<Group[]>([]);
  const [name, setName] = useState("");
  const [destinationPath, setDestinationPath] = useState("");
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function refresh() {
    invoke<Group[]>("list_groups")
      .then(setGroups)
      .catch((err) => setError(String(err)));
  }

  useEffect(refresh, []);

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setDestinationPath(selected);
    }
  }

  async function createGroup(event: FormEvent) {
    event.preventDefault();
    if (!name.trim() || !destinationPath.trim()) return;
    setCreating(true);
    setError(null);
    try {
      await invoke("create_group", {
        name: name.trim(),
        destinationPath: destinationPath.trim(),
      });
      setName("");
      setDestinationPath("");
      refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setCreating(false);
    }
  }

  return (
    <>
      <header className="panel-header">
        <h1>Groups</h1>
        <p>
          Each group is a destination folder. Once a group exists, it shows up as a button on the
          Floating Card so a download can be filed straight there.
        </p>
      </header>

      <form onSubmit={(e) => void createGroup(e)} className="group-form">
        <input
          className="group-form-name"
          placeholder="Group name (e.g. Design)"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <div className="group-form-path">
          <input
            placeholder="Destination folder"
            value={destinationPath}
            onChange={(e) => setDestinationPath(e.target.value)}
          />
          <button type="button" className="btn-secondary" onClick={() => void pickFolder()}>
            Browse…
          </button>
        </div>
        <button type="submit" className="btn-primary" disabled={creating}>
          {creating ? "Creating…" : "Create group"}
        </button>
      </form>
      {error && <p className="form-error">{error}</p>}

      {groups.length === 0 ? (
        <EmptyState icon={<GroupsIcon width={28} height={28} />} text="No groups yet." />
      ) : (
        <ul className="group-list">
          {groups.map((group) => (
            <li key={group.id} className="group-card">
              <GroupsIcon width={18} height={18} />
              <div>
                <div className="group-name">{group.name}</div>
                <div className="group-path">{group.destination_path}</div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}

function TemporaryPanel() {
  const [files, setFiles] = useState<FileRecord[]>([]);
  const [groups, setGroups] = useState<Group[]>([]);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function refresh() {
    invoke<FileRecord[]>("list_temporary")
      .then(setFiles)
      .catch((err) => setError(String(err)));
  }

  useEffect(() => {
    refresh();
    invoke<Group[]>("list_groups")
      .then(setGroups)
      .catch(() => setGroups([]));

    const unlisten = listen("temporary-expired", refresh);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function keepLonger(fileId: string) {
    setBusyId(fileId);
    try {
      await invoke("mark_temporary", { fileId });
      refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  async function moveToGroup(fileId: string, groupId: string) {
    if (!groupId) return;
    setBusyId(fileId);
    try {
      await invoke("assign_group", { fileId, groupId });
      refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  async function moveToRecycleBin(fileId: string, name: string) {
    if (!window.confirm(`Move "${name}" to the Recycle Bin?`)) return;
    setBusyId(fileId);
    try {
      await invoke("move_to_recycle_bin", { fileId });
      refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusyId(null);
    }
  }

  return (
    <>
      <header className="panel-header">
        <h1>Temporary</h1>
        <p>
          Files marked Temporary stick around for a while, then land here to be kept longer, filed
          into a group, or sent to the Recycle Bin. Nothing here is ever auto-deleted.
        </p>
      </header>
      {error && <p className="form-error">{error}</p>}

      {files.length === 0 ? (
        <EmptyState
          icon={<TemporaryIcon width={28} height={28} />}
          text="Nothing marked Temporary right now."
        />
      ) : (
        <ul className="file-list">
          {files.map((file) => (
            <li key={file.id} className="file-row temporary-row">
              <span className="file-name" title={file.current_name}>
                {file.current_name}
              </span>
              <span className="file-size">{formatSize(file.size_bytes)}</span>
              <span className={`status-badge status-badge-${file.status}`}>
                {file.status === "cleanup_ready"
                  ? "ready for cleanup"
                  : file.expires_at
                    ? `expires ${formatRelative(file.expires_at)}`
                    : "temporary"}
              </span>
              <div className="temporary-actions">
                <button
                  className="btn-link"
                  disabled={busyId === file.id}
                  onClick={() => void keepLonger(file.id)}
                >
                  Keep 7 more days
                </button>
                <select
                  disabled={busyId === file.id || groups.length === 0}
                  defaultValue=""
                  onChange={(e) => void moveToGroup(file.id, e.target.value)}
                >
                  <option value="" disabled>
                    Move to group…
                  </option>
                  {groups.map((group) => (
                    <option key={group.id} value={group.id}>
                      {group.name}
                    </option>
                  ))}
                </select>
                <button
                  className="btn-link btn-link-danger"
                  disabled={busyId === file.id}
                  onClick={() => void moveToRecycleBin(file.id, file.current_name)}
                >
                  Recycle Bin
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}

function EmptyState({ icon, text }: { icon: ReactNode; text: string }) {
  return (
    <div className="empty-state">
      {icon}
      <p>{text}</p>
    </div>
  );
}
