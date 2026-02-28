import { findRun, statusColors, ciConfig } from "../data/runs";
import type { Route } from "./+types/run-detail";

export function meta({ params }: Route.MetaArgs) {
  const run = findRun(params.id);
  return [{ title: run ? `${run.title} — Arc` : "Run — Arc" }];
}

export default function RunDetail({ params }: Route.ComponentProps) {
  const run = findRun(params.id);

  if (!run) {
    return <p className="py-8 text-center text-sm text-navy-600">Run not found.</p>;
  }

  const colors = statusColors[run.status];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-ice-100">{run.title}</h2>
        <div className="mt-2 flex items-center gap-3 text-sm">
          <span className="flex items-center gap-1.5">
            <span className={`size-2 rounded-full ${colors.dot}`} />
            <span className={`font-medium ${colors.text}`}>{run.statusLabel}</span>
          </span>
          <span className="font-mono text-xs text-navy-600">{run.repo}</span>
          {run.elapsed && (
            <span className={`font-mono text-xs ${run.elapsedWarning ? "text-amber" : "text-navy-600"}`}>{run.elapsed}</span>
          )}
        </div>
      </div>

      <div className="rounded-lg border border-white/[0.06] bg-navy-800/80 p-5 space-y-4">
        {run.resources && (
          <div className="flex items-center justify-between">
            <span className="text-xs text-navy-600">Resources</span>
            <span className="font-mono text-sm text-ice-300">{run.resources}</span>
          </div>
        )}
        {(run.additions != null || run.deletions != null) && (
          <div className="flex items-center justify-between">
            <span className="text-xs text-navy-600">Changes</span>
            <span className="font-mono text-sm">
              {run.additions != null && <span className="text-mint">+{run.additions.toLocaleString()}</span>}
              {run.additions != null && run.deletions != null && <span className="text-navy-600"> / </span>}
              {run.deletions != null && <span className="text-coral">-{run.deletions.toLocaleString()}</span>}
            </span>
          </div>
        )}
        {run.number != null && (
          <div className="flex items-center justify-between">
            <span className="text-xs text-navy-600">Pull Request</span>
            <span className="font-mono text-sm text-ice-300">#{run.number}</span>
          </div>
        )}
        {run.ci && (
          <div className="flex items-center justify-between">
            <span className="text-xs text-navy-600">CI</span>
            <span className={`font-mono text-sm ${ciConfig[run.ci].text}`}>{ciConfig[run.ci].label}</span>
          </div>
        )}
      </div>
    </div>
  );
}
