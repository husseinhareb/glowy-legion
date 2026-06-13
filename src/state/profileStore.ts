import { useCallback, useState } from "react";

import { listBuiltinProfiles } from "../api/profileApi";
import type { LightingProfile } from "../domain/profile";

export function useProfileStore() {
  const [profiles, setProfiles] = useState<LightingProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadProfiles = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const builtinProfiles = await listBuiltinProfiles();
      setProfiles(builtinProfiles);
      return builtinProfiles;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setError(message);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { profiles, loading, error, loadProfiles };
}
