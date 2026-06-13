import type { AppInfo } from "../domain/app";
import { invokeCommand } from "./tauriClient";

export function getAppInfo(): Promise<AppInfo> {
  return invokeCommand<AppInfo>("get_app_info");
}
