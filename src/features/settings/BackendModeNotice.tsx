import { Notice } from "../../shared/components/Notice";
import { backendModeNotice } from "../../app/constants";

interface BackendModeNoticeProps {
  backendMode?: string;
}

export function BackendModeNotice({ backendMode = "mock" }: BackendModeNoticeProps) {
  return (
    <Notice tone={backendMode === "lenovo-hid" ? "warning" : "info"}>
      {backendModeNotice(backendMode)}
    </Notice>
  );
}
