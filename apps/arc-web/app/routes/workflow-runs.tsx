import { allRunsFlat, ciConfig, statusColors } from "../data/runs";
import type { CiStatus, RunWithStatus } from "../data/runs";

const runs = allRunsFlat();

function CiBadge({ status }: { status: CiStatus }) {
  const config = ciConfig[status];
  return (
    <span className={`inline-flex items-center gap-1.5 font-mono text-xs ${config.text}`}>
      <span className={`size-1.5 rounded-full ${config.dot}`} />
      {config.label}
    </span>
  );
}

function RunRow({ run }: { run: RunWithStatus }) {
  const colors = statusColors[run.status];
  return (
    <div className="flex items-center gap-4 rounded-lg border border-white/[0.06] bg-navy-800/80 px-4 py-3 transition-all duration-200 hover:border-white/[0.12] hover:bg-navy-800">
      <span className="flex items-center gap-2 min-w-[80px]">
        <span className={`size-2 shrink-0 rounded-full ${colors.dot}`} />
        <span className={`text-xs font-medium ${colors.text}`}>{run.statusLabel}</span>
      </span>

      <span className="font-mono text-xs font-medium text-teal-500 min-w-[110px]">
        {run.repo}
        {run.number != null && (
          <span className="text-navy-600"> #{run.number}</span>
        )}
      </span>

      <span className="flex-1 truncate text-sm text-ice-100">{run.title}</span>

      {run.additions != null && run.deletions != null && (
        <span className="hidden sm:flex items-center gap-2 font-mono text-xs">
          <span className="text-mint">+{run.additions.toLocaleString()}</span>
          <span className="text-coral">-{run.deletions.toLocaleString()}</span>
        </span>
      )}

      {run.ci != null && (
        <span className="hidden md:block min-w-[120px]">
          <CiBadge status={run.ci} />
        </span>
      )}

      {run.comments != null && run.comments > 0 && (
        <span className="hidden sm:inline-flex items-center gap-1 font-mono text-xs text-navy-600">
          <svg viewBox="0 0 16 16" fill="currentColor" className="size-3" aria-hidden="true">
            <path d="M1 2.75C1 1.784 1.784 1 2.75 1h10.5c.966 0 1.75.784 1.75 1.75v7.5A1.75 1.75 0 0 1 13.25 12H9.06l-2.573 2.573A1.458 1.458 0 0 1 4 13.543V12H2.75A1.75 1.75 0 0 1 1 10.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h4.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z" />
          </svg>
          {run.comments}
        </span>
      )}

      <span className={`font-mono text-xs min-w-[60px] text-right ${run.elapsedWarning ? "text-amber" : "text-navy-600"}`}>
        {run.elapsed}
      </span>
    </div>
  );
}

export default function WorkflowRuns() {
  return (
    <div className="space-y-2">
      {runs.map((run) => (
        <RunRow key={`${run.repo}-${run.number ?? run.title}`} run={run} />
      ))}
    </div>
  );
}
