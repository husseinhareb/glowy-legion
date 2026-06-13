import type { KeyboardState } from "../domain/lighting";
import { invokeCommand } from "./tauriClient";

export function getKeyboardState(): Promise<KeyboardState> {
  return invokeCommand<KeyboardState>("get_keyboard_state");
}

export function setKeyboardState(state: KeyboardState): Promise<KeyboardState> {
  return invokeCommand<KeyboardState>("set_keyboard_state", { state });
}

export function turnBacklightOff(): Promise<KeyboardState> {
  return invokeCommand<KeyboardState>("turn_backlight_off");
}

export function sendSafeTestPayload(): Promise<KeyboardState> {
  return invokeCommand<KeyboardState>("send_safe_test_payload");
}
