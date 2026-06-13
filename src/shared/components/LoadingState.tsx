interface LoadingStateProps {
  label?: string;
}

export function LoadingState({ label = "Loading" }: LoadingStateProps) {
  return (
    <div className="loading-state" role="status">
      <span className="loading-state__dot" />
      {label}
    </div>
  );
}
