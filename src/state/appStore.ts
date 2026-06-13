import { useCallback, useState } from "react";

import { getAppInfo } from "../api/appApi";
import type { AppInfo } from "../domain/app";

export function useAppStore() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAppInfo = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await getAppInfo();
      setAppInfo(info);
      return info;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setError(message);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { appInfo, loading, error, loadAppInfo };
}
