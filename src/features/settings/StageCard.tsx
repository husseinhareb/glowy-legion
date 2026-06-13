import type { ReactNode } from "react";

import { Card } from "../../shared/components/Card";
import { StatusBadge } from "../../shared/components/StatusBadge";

export type StageTone = "ok" | "warn" | "danger";

interface StageCardProps {
  index: number;
  title: string;
  statusLabel: string;
  statusTone: StageTone;
  description?: string;
  children?: ReactNode;
}

export function StageCard({
  index,
  title,
  statusLabel,
  statusTone,
  description,
  children,
}: StageCardProps) {
  return (
    <Card className="stage-card">
      <div className="card__header">
        <div>
          <p className="eyebrow">Stage {index}</p>
          <h2>{title}</h2>
        </div>
        <StatusBadge label={statusLabel} tone={statusTone} />
      </div>
      {description && <p className="stage-card__description">{description}</p>}
      {children}
    </Card>
  );
}
