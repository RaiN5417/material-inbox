import { useEffect, useRef, useState } from "react";

export interface ContextMenuItem {
  label: string;
  onSelect?: () => void;
  danger?: boolean;
  disabled?: boolean;
  submenu?: { label: string; onSelect: () => void }[];
}

// A small hand-rolled right-click menu — native WebView2 gives file cards a
// generic browser menu (Back/Reload/Inspect) with nothing useful on it, so
// this replaces it via `e.preventDefault()` at the call site.
export function ContextMenu({
  x,
  y,
  items,
  onClose,
}: {
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [openSubmenu, setOpenSubmenu] = useState<number | null>(null);
  const [pos, setPos] = useState({ x, y });

  useEffect(() => {
    const menu = menuRef.current;
    if (!menu) return;
    const rect = menu.getBoundingClientRect();
    const clampedX = Math.min(x, window.innerWidth - rect.width - 8);
    const clampedY = Math.min(y, window.innerHeight - rect.height - 8);
    setPos({ x: Math.max(8, clampedX), y: Math.max(8, clampedY) });
    // Only clamp once, right after the menu (and its current submenu) mount
    // with real dimensions — not on every render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    function onPointerDown(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) onClose();
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("mousedown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("mousedown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("blur", onClose);
    };
  }, [onClose]);

  return (
    <div ref={menuRef} className="context-menu" style={{ left: pos.x, top: pos.y }}>
      {items.map((item, i) => (
        <div
          key={i}
          className="context-menu-item-wrap"
          onMouseEnter={() => setOpenSubmenu(item.submenu ? i : null)}
        >
          <button
            className={`context-menu-item ${item.danger ? "danger" : ""}`}
            disabled={item.disabled}
            onClick={() => {
              if (item.submenu) return;
              item.onSelect?.();
              onClose();
            }}
          >
            {item.label}
            {item.submenu && <span className="context-menu-caret">›</span>}
          </button>
          {item.submenu && openSubmenu === i && (
            <div className="context-menu context-submenu">
              {item.submenu.map((sub, j) => (
                <button
                  key={j}
                  className="context-menu-item"
                  onClick={() => {
                    sub.onSelect();
                    onClose();
                  }}
                >
                  {sub.label}
                </button>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
