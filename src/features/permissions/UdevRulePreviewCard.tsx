import type { UdevRulePreview } from "../../domain/diagnostics";
import { Button } from "../../shared/components/Button";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";

interface UdevRulePreviewCardProps {
  preview: UdevRulePreview | null;
  loading: boolean;
  error: string | null;
  onPreview: () => void;
  onCopyRule: () => void;
  onCopyManual: () => void;
}

export function UdevRulePreviewCard({
  preview,
  loading,
  error,
  onPreview,
  onCopyRule,
  onCopyManual,
}: UdevRulePreviewCardProps) {
  const available = preview?.available ?? false;

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">udev rule</p>
          <h3>Rule preview</h3>
        </div>
        <Button disabled={loading} onClick={onPreview}>
          Preview udev rule
        </Button>
      </div>

      {error && <Notice tone="error">{error}</Notice>}

      {preview && !available && (
        <Notice tone="info">{preview.explanation}</Notice>
      )}

      {preview && available && (
        <>
          <Notice tone="info">{preview.explanation}</Notice>
          <div className="payload-preview">
            <span>
              {preview.filename} · {preview.vendor_id}:{preview.product_id}
            </span>
            <code>{preview.rule}</code>
          </div>
          <div className="action-row">
            <Button disabled={!available} onClick={onCopyRule}>
              Copy udev rule
            </Button>
            <Button disabled={!available} onClick={onCopyManual}>
              Copy manual install commands
            </Button>
          </div>
          {preview.warnings.map((warning) => (
            <Notice key={warning} tone="warning">
              {warning}
            </Notice>
          ))}
        </>
      )}
    </Card>
  );
}
