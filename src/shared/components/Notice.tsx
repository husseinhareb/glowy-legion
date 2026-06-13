import type { ReactNode } from "react";

interface NoticeProps {
  children: ReactNode;
  tone?: "info" | "success" | "error" | "warning";
}

export function Notice({ children, tone = "info" }: NoticeProps) {
  return <div className={`notice notice--${tone}`}>{children}</div>;
}
