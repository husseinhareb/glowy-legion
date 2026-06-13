import { useCallback, useState } from "react";

export type ToastKind = "success" | "error" | "info";

export interface ToastMessage {
  kind: ToastKind;
  message: string;
}

export function useToast() {
  const [toast, setToast] = useState<ToastMessage | null>(null);

  const showToast = useCallback((kind: ToastKind, message: string) => {
    setToast({ kind, message });
  }, []);

  const clearToast = useCallback(() => {
    setToast(null);
  }, []);

  return { toast, showToast, clearToast };
}
