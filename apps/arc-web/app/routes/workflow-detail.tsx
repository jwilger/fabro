import { useState } from "react";
import { useParams } from "react-router";
import type { Route } from "./+types/workflow-detail";

const workflowData: Record<string, { title: string; description: string; filename: string }> = {
  fix_build: {
    title: "Fix Build",
    filename: "fix_build.dot",
    description: "Automatically diagnoses and fixes CI build failures by analyzing error logs, identifying root causes, and applying targeted code changes.",
  },
  implement: {
    title: "Implement Feature",
    filename: "implement.dot",
    description: "Generates production-ready code from a technical blueprint, including tests, documentation, and a pull request ready for review.",
  },
  sync_drift: {
    title: "Sync Drift",
    filename: "sync_drift.dot",
    description: "Detects configuration and code drift between environments, then generates reconciliation patches to bring everything back in sync.",
  },
  expand: {
    title: "Expand Product",
    filename: "expand.dot",
    description: "Evolves the product by analyzing usage patterns and specifications to propose and implement incremental improvements.",
  },
};

const tabs = ["Definition", "Diagram", "Runs"] as const;
type Tab = (typeof tabs)[number];

export function meta({ params }: Route.MetaArgs) {
  const workflow = workflowData[params.name ?? ""];
  const title = workflow?.title ?? params.name;
  return [{ title: `${title} — Arc` }];
}

export default function WorkflowDetail() {
  const { name } = useParams();
  const workflow = workflowData[name ?? ""];
  const [activeTab, setActiveTab] = useState<Tab>("Definition");

  if (workflow == null) {
    return <p className="text-sm text-ice-300">Workflow not found.</p>;
  }

  return (
    <div>
      <div className="mb-6">
        <div className="flex items-center gap-3">
          <h2 className="text-xl font-semibold text-white">{workflow.title}</h2>
          <span className="font-mono text-xs text-navy-600">{workflow.filename}</span>
        </div>
        <p className="mt-2 text-sm leading-relaxed text-ice-300">{workflow.description}</p>
      </div>

      <div className="border-b border-white/[0.06]">
        <nav className="-mb-px flex gap-6">
          {tabs.map((tab) => (
            <button
              key={tab}
              type="button"
              onClick={() => setActiveTab(tab)}
              className={`border-b-2 pb-3 text-sm font-medium transition-colors ${
                activeTab === tab
                  ? "border-teal-500 text-white"
                  : "border-transparent text-navy-600 hover:border-white/10 hover:text-ice-300"
              }`}
            >
              {tab}
            </button>
          ))}
        </nav>
      </div>

      <div className="mt-6">
        {activeTab === "Definition" && (
          <p className="text-sm text-navy-600">Workflow definition will appear here.</p>
        )}
        {activeTab === "Diagram" && (
          <p className="text-sm text-navy-600">Workflow diagram will appear here.</p>
        )}
        {activeTab === "Runs" && (
          <p className="text-sm text-navy-600">Recent runs will appear here.</p>
        )}
      </div>
    </div>
  );
}
