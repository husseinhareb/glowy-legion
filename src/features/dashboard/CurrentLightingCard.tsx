import type { KeyboardState } from "../../domain/lighting";
import { Card } from "../../shared/components/Card";
import { StatusBadge } from "../../shared/components/StatusBadge";
import { rgbToCss } from "../../shared/utils/color";
import { formatDirection, formatEffect } from "../../shared/utils/formatting";

interface CurrentLightingCardProps {
  state: KeyboardState | null;
}

export function CurrentLightingCard({ state }: CurrentLightingCardProps) {
  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Current lighting</p>
          <h2>{state ? formatEffect(state.effect) : "Unknown"}</h2>
        </div>
        <StatusBadge
          label={state?.enabled ? "Enabled" : "Off"}
          tone={state?.enabled ? "ok" : "neutral"}
        />
      </div>
      <div className="lighting-preview">
        <span
          className="color-chip color-chip--large"
          style={{ background: rgbToCss(state?.primary_color ?? null) }}
        />
        <div>
          <strong>{state?.brightness ?? 0}% brightness</strong>
          <span>{state ? formatDirection(state.direction) : "No direction"}</span>
        </div>
      </div>
      <dl className="detail-grid">
        <div>
          <dt>Speed</dt>
          <dd>{state?.speed ?? 0}</dd>
        </div>
        <div>
          <dt>Secondary color</dt>
          <dd>{state?.secondary_color ? "Available" : "None"}</dd>
        </div>
      </dl>
    </Card>
  );
}
