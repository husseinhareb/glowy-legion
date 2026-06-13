import { useCallback, useState } from "react";

export interface AsyncState<TValue> {
  value: TValue | null;
  loading: boolean;
  error: string | null;
}

export function useAsync<TValue>() {
  const [state, setState] = useState<AsyncState<TValue>>({
    value: null,
    loading: false,
    error: null,
  });

  const run = useCallback(async (task: () => Promise<TValue>) => {
    setState((current) => ({ ...current, loading: true, error: null }));

    try {
      const value = await task();
      setState({ value, loading: false, error: null });
      return value;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : String(error ?? "Unknown error");
      setState((current) => ({ ...current, loading: false, error: message }));
      throw error;
    }
  }, []);

  return { ...state, run };
}
