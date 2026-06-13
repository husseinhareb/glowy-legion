import type { PermissionSetupResult } from "../../domain/permissions";
import { Card } from "../../shared/components/Card";
import { Notice } from "../../shared/components/Notice";
import { StatusBadge } from "../../shared/components/StatusBadge";

interface PermissionSetupResultCardProps {
  result: PermissionSetupResult | null;
}

export function PermissionSetupResultCard({
  result,
}: PermissionSetupResultCardProps) {
  if (!result) {
    return null;
  }

  return (
    <Card>
      <div className="card__header">
        <div>
          <p className="eyebrow">Last action · {result.action}</p>
          <h3>{result.message}</h3>
        </div>
        <StatusBadge
          label={result.success ? "Success" : "Failed"}
          tone={result.success ? "ok" : "danger"}
        />
      </div>

      {result.next_steps.length > 0 && (
        <div className="next-steps">
          <span>Next steps</span>
          <ul>
            {result.next_steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ul>
        </div>
      )}

      {result.warnings.map((warning) => (
        <Notice key={warning} tone="warning">
          {warning}
        </Notice>
      ))}

      {result.stdout && (
        <details className="process-output">
          <summary>stdout</summary>
          <code>{result.stdout}</code>
        </details>
      )}
      {result.stderr && (
        <details className="process-output">
          <summary>stderr</summary>
          <code>{result.stderr}</code>
        </details>
      )}
    </Card>
  );
}
