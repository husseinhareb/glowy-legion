import { useCallback, useState } from "react";

import { detectKeyboardDevice } from "../api/deviceApi";
import {
  getKeyboardState,
  setKeyboardState,
  turnBacklightOff,
} from "../api/lightingApi";
import type { DeviceInfo } from "../domain/device";
import type { KeyboardState } from "../domain/lighting";

export interface KeyboardStore {
  device: DeviceInfo | null;
  keyboardState: KeyboardState | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  applyState: (state: KeyboardState) => Promise<KeyboardState>;
  turnOff: () => Promise<KeyboardState>;
  replaceKeyboardState: (state: KeyboardState) => void;
}

export function useKeyboardStore(): KeyboardStore {
  const [device, setDevice] = useState<DeviceInfo | null>(null);
  const [keyboardState, setKeyboardStateValue] = useState<KeyboardState | null>(
    null,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const withLoading = useCallback(async <TResult,>(
    task: () => Promise<TResult>,
  ) => {
    setLoading(true);
    setError(null);

    try {
      return await task();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setError(message);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  const refresh = useCallback(async () => {
    await withLoading(async () => {
      const [detectedDevice, currentState] = await Promise.all([
        detectKeyboardDevice(),
        getKeyboardState(),
      ]);
      setDevice(detectedDevice);
      setKeyboardStateValue(currentState);
    });
  }, [withLoading]);

  const applyState = useCallback(
    async (state: KeyboardState) =>
      withLoading(async () => {
        const updated = await setKeyboardState(state);
        setKeyboardStateValue(updated);
        return updated;
      }),
    [withLoading],
  );

  const turnOff = useCallback(
    async () =>
      withLoading(async () => {
        const updated = await turnBacklightOff();
        setKeyboardStateValue(updated);
        return updated;
      }),
    [withLoading],
  );

  return {
    device,
    keyboardState,
    loading,
    error,
    refresh,
    applyState,
    turnOff,
    replaceKeyboardState: setKeyboardStateValue,
  };
}
