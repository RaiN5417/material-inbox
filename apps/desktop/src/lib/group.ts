// Mirrors crates/domain/src/group.rs Group's serde output.
export interface Group {
  id: string;
  name: string;
  destination_path: string | null;
  icon: string | null;
  is_pinned: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}
