import type { Route } from "./+types/settings";

export function meta({}: Route.MetaArgs) {
  return [{ title: "Settings — Arc" }];
}

export const handle = { hideHeader: true };

interface SettingField {
  key: string;
  label: string;
  value: string;
  type: "text" | "select" | "toggle";
  options?: string[];
  description?: string;
}

interface SettingGroup {
  id: string;
  name: string;
  description: string;
  fields: SettingField[];
}

const settingGroups: SettingGroup[] = [
  {
    id: "general",
    name: "General",
    description: "Core platform settings and defaults.",
    fields: [
      { key: "org_name", label: "Organization name", value: "Acme Corp", type: "text" },
      { key: "default_branch", label: "Default branch", value: "main", type: "text" },
      { key: "timezone", label: "Timezone", value: "America/New_York", type: "select", options: ["America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles", "UTC", "Europe/London", "Europe/Berlin", "Asia/Tokyo"] },
      { key: "auto_cancel", label: "Auto-cancel superseded runs", value: "true", type: "toggle" },
    ],
  },
  {
    id: "git",
    name: "Git & VCS",
    description: "Version control integration and repository settings.",
    fields: [
      { key: "github_org", label: "GitHub organization", value: "acme-corp", type: "text" },
      { key: "clone_protocol", label: "Clone protocol", value: "SSH", type: "select", options: ["SSH", "HTTPS"] },
      { key: "auto_merge", label: "Auto-merge when checks pass", value: "false", type: "toggle" },
      { key: "delete_branch", label: "Delete branch after merge", value: "true", type: "toggle" },
      { key: "commit_signing", label: "Require commit signing", value: "false", type: "toggle" },
    ],
  },
  {
    id: "compute",
    name: "Compute",
    description: "Resource allocation and execution environment.",
    fields: [
      { key: "default_cpu", label: "Default CPU", value: "4", type: "select", options: ["2", "4", "8", "16"] },
      { key: "default_memory", label: "Default memory", value: "8 GB", type: "select", options: ["4 GB", "8 GB", "16 GB", "32 GB"] },
      { key: "max_parallel", label: "Max parallel runs", value: "10", type: "text" },
      { key: "timeout_minutes", label: "Run timeout (minutes)", value: "120", type: "text" },
      { key: "gpu_enabled", label: "GPU acceleration", value: "false", type: "toggle" },
    ],
  },
  {
    id: "notifications",
    name: "Notifications",
    description: "Alerts and notification delivery preferences.",
    fields: [
      { key: "slack_webhook", label: "Slack webhook URL", value: "https://hooks.slack.com/services/T00/B00/xxxx", type: "text" },
      { key: "notify_on_failure", label: "Notify on failure", value: "true", type: "toggle" },
      { key: "notify_on_success", label: "Notify on success", value: "false", type: "toggle" },
      { key: "notify_on_approval", label: "Notify on approval needed", value: "true", type: "toggle" },
      { key: "email_digest", label: "Daily email digest", value: "false", type: "toggle" },
    ],
  },
  {
    id: "security",
    name: "Security",
    description: "Access control and security policies.",
    fields: [
      { key: "sso_provider", label: "SSO provider", value: "Okta", type: "select", options: ["None", "Okta", "Azure AD", "Google Workspace", "OneLogin"] },
      { key: "mfa_required", label: "Require MFA", value: "true", type: "toggle" },
      { key: "session_timeout", label: "Session timeout", value: "8 hours", type: "select", options: ["1 hour", "4 hours", "8 hours", "24 hours", "7 days"] },
      { key: "audit_log", label: "Audit logging", value: "true", type: "toggle" },
      { key: "ip_allowlist", label: "IP allowlist", value: "", type: "text", description: "Comma-separated CIDRs. Leave empty to allow all." },
    ],
  },
];

function ToggleSwitch({ enabled }: { enabled: boolean }) {
  return (
    <button
      type="button"
      className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors ${enabled ? "bg-teal-500" : "bg-white/[0.08]"}`}
      role="switch"
      aria-checked={enabled}
    >
      <span
        className={`pointer-events-none inline-block size-4 rounded-full bg-white shadow transition-transform ${enabled ? "translate-x-4" : "translate-x-0"}`}
      />
    </button>
  );
}

function SettingRow({ field }: { field: SettingField }) {
  return (
    <div className="flex items-center justify-between gap-8 py-3.5">
      <div className="min-w-0">
        <p className="text-sm text-ice-100">{field.label}</p>
        {field.description != null && (
          <p className="mt-0.5 text-xs text-navy-600">{field.description}</p>
        )}
      </div>

      <div className="shrink-0">
        {field.type === "toggle" ? (
          <ToggleSwitch enabled={field.value === "true"} />
        ) : field.type === "select" ? (
          <select
            defaultValue={field.value}
            className="appearance-none rounded-md border border-white/[0.06] bg-navy-800/80 py-1.5 pl-3 pr-8 text-sm text-ice-100 outline-none transition-colors focus:border-teal-500/40 focus:ring-0"
          >
            {field.options?.map((opt) => (
              <option key={opt} value={opt}>{opt}</option>
            ))}
          </select>
        ) : (
          <input
            type="text"
            defaultValue={field.value}
            className="w-64 rounded-md border border-white/[0.06] bg-navy-800/80 px-3 py-1.5 text-sm text-ice-100 placeholder-navy-600 outline-none transition-colors focus:border-teal-500/40 focus:ring-0"
          />
        )}
      </div>
    </div>
  );
}

function SettingsSection({ group }: { group: SettingGroup }) {
  return (
    <section className="rounded-lg border border-white/[0.06] bg-navy-800/50">
      <div className="border-b border-white/[0.06] px-5 py-4">
        <h2 className="text-sm font-semibold text-ice-100">{group.name}</h2>
        <p className="mt-0.5 text-xs text-navy-600">{group.description}</p>
      </div>
      <div className="divide-y divide-white/[0.06] px-5">
        {group.fields.map((field) => (
          <SettingRow key={field.key} field={field} />
        ))}
      </div>
    </section>
  );
}

export default function Settings() {
  return (
    <div className="space-y-6">
      {settingGroups.map((group) => (
        <SettingsSection key={group.id} group={group} />
      ))}
    </div>
  );
}
