import type { DeviceInfo } from "../domain/device";
import { invokeCommand } from "./tauriClient";

export function detectKeyboardDevice(): Promise<DeviceInfo> {
  return invokeCommand<DeviceInfo>("detect_keyboard_device");
}
