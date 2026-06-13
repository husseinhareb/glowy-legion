import type { DiagnosticsReport } from "../domain/diagnostics";
import { invokeCommand } from "./tauriClient";

export function runDiagnostics(): Promise<DiagnosticsReport> {
  return invokeCommand<DiagnosticsReport>("run_diagnostics");
}
