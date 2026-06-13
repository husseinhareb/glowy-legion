import type { DeviceCapabilities } from "../../domain/device";
import type { KeyboardState } from "../../domain/lighting";
import type { LightingProfile } from "../../domain/profile";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { rgbToCss } from "../../shared/utils/color";
import { formatEffect } from "../../shared/utils/formatting";
import { canApplyState } from "../../shared/utils/validation";

interface ProfileCardProps {
  profile: LightingProfile;
  capabilities: DeviceCapabilities | null;
  loading: boolean;
  onApply: (profileId: string) => void;
}

export function ProfileCard({
  profile,
  capabilities,
  loading,
  onApply,
}: ProfileCardProps) {
  // The backend is the final authority on compatibility; the client-side check
  // is only a fallback so the UI degrades gracefully if it is ever missing.
  const supported =
    profile.compatibility?.supported ??
    canApplyState(profile.state, capabilities);
  const reasons = profile.compatibility?.reasons ?? [];

  return (
    <Card className="profile-card">
      <div className="profile-card__header">
        <span
          className="color-chip"
          style={{ background: profileGradient(profile.state) }}
        />
        <StatusBadge
          label={supported ? formatEffect(profile.state.effect) : "Unsupported"}
          tone={supported ? "ok" : "warn"}
        />
      </div>
      <div>
        <h2>{profile.name}</h2>
        <p>{profile.description}</p>
      </div>
      {!supported && reasons.length > 0 && (
        <Notice tone="warning">
          {reasons.join(" ")}
        </Notice>
      )}
      <Button
        disabled={loading || !supported}
        fullWidth
        variant="primary"
        onClick={() => onApply(profile.id)}
      >
        {supported ? "Apply profile" : "Unsupported on this device"}
      </Button>
    </Card>
  );
}

function profileGradient(state: KeyboardState): string {
  const primary = rgbToCss(state.primary_color);
  const secondary = rgbToCss(state.secondary_color ?? state.primary_color);

  return `linear-gradient(135deg, ${primary}, ${secondary})`;
}
