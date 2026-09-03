import { useEffect, type ReactNode } from "react";

// A minimal hand-rolled dialog primitive — backdrop + centered panel,
// Escape/backdrop-click to close — rather than pulling in a dialog library
// for the one popup this app needs (spec section 48: justify every new
// dependency).
export function Modal({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div
        className="modal-panel"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h2>{title}</h2>
        </div>
        {children}
      </div>
    </div>
  );
}
