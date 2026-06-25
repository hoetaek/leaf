import { useEffect, useState } from "react";

// Minimal app-wide toast: a module-level pub/sub so any handler can call
// showToast(...) without prop-drilling or context. A single <Toaster/> mounted
// in App subscribes and renders the latest message, auto-hiding after a beat.
// Reusable for any transient confirmation (e.g. a future fall action).

export interface ToastEvent {
  id: number;
  message: string;
}

type ToastListener = (event: ToastEvent) => void;

const listeners = new Set<ToastListener>();
let sequence = 0;

export function showToast(message: string): void {
  const event: ToastEvent = { id: ++sequence, message };
  for (const listener of listeners) listener(event);
}

export function subscribeToast(listener: ToastListener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

const TOAST_VISIBLE_MS = 1300;

export function Toaster() {
  const [toast, setToast] = useState<ToastEvent | null>(null);

  useEffect(() => subscribeToast(setToast), []);

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), TOAST_VISIBLE_MS);
    return () => clearTimeout(timer);
  }, [toast]);

  // The wrapper is the persistent aria-live region; the pill is the visible toast.
  return (
    <div aria-live="polite" aria-atomic="true">
      {toast && (
        <div className="toast show" key={toast.id}>
          {toast.message}
        </div>
      )}
    </div>
  );
}
