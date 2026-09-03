// Mirrors crates/domain/src/operation.rs Operation's serde output.
export interface Operation {
  id: string;
  file_id: string;
  operation_type: string;
  source_path: string | null;
  destination_path: string | null;
  group_id: string | null;
  status: string;
  created_at: string;
  completed_at: string | null;
  undone_at: string | null;
  error_code: string | null;
  error_message: string | null;
}

export function fileNameFromPath(path: string | null): string {
  if (!path) return "(unknown)";
  const normalized = path.replace(/\\/g, "/");
  return normalized.substring(normalized.lastIndexOf("/") + 1);
}
