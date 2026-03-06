export type CiStatus = "passing" | "failing" | "pending";

export type CheckStatus = "success" | "failure" | "skipped" | "pending" | "queued";

export interface CheckRun {
  name: string;
  status: CheckStatus;
  duration?: string;
}

export interface RunItem {
  id: string;
  repo: string;
  title: string;
  workflow: string;
  number?: number;
  additions?: number;
  deletions?: number;
  checks?: CheckRun[];
  elapsed?: string;
  elapsedWarning?: boolean;
  resources?: string;
  actionDisabled?: boolean;
  comments?: number;
  question?: string;
  sandboxId?: string;
}

export type ColumnStatus = "working" | "pending" | "review" | "merge";

export interface RunWithStatus extends RunItem {
  status: ColumnStatus;
  statusLabel: string;
}

export function deriveCiStatus(checks: CheckRun[]): CiStatus {
  if (checks.some((c) => c.status === "failure")) return "failing";
  if (checks.some((c) => c.status === "pending" || c.status === "queued")) return "pending";
  return "passing";
}

export const statusColors: Record<ColumnStatus, { dot: string; text: string }> = {
  working: { dot: "bg-teal-500", text: "text-teal-500" },
  pending: { dot: "bg-amber", text: "text-amber" },
  review: { dot: "bg-mint", text: "text-mint" },
  merge: { dot: "bg-teal-300", text: "text-teal-300" },
};

export const ciConfig: Record<CiStatus, { label: string; dot: string; text: string }> = {
  passing: { label: "Passing", dot: "bg-mint", text: "text-mint" },
  failing: { label: "Changes needed", dot: "bg-coral", text: "text-coral" },
  pending: { label: "Pending", dot: "bg-amber", text: "text-amber" },
};
