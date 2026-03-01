import { useState } from "react";
import { ChevronDownIcon, Cog6ToothIcon } from "@heroicons/react/24/outline";
import { MultiFileDiff } from "@pierre/diffs/react";

export const handle = { wide: true };

const checkpoints = [
  { id: "all", label: "All changes" },
  { id: "cp-4", label: "Checkpoint 4 — Apply Changes" },
  { id: "cp-3", label: "Checkpoint 3 — Review Changes" },
  { id: "cp-2", label: "Checkpoint 2 — Propose Changes" },
  { id: "cp-1", label: "Checkpoint 1 — Detect Drift" },
];

const files = [
  {
    oldFile: {
      name: "src/commands/run.ts",
      contents: `import { parseArgs } from "node:util";
import { loadConfig } from "../config.js";
import { execute } from "../executor.js";

interface RunOptions {
  config: string;
  dryRun: boolean;
}

export async function run(argv: string[]) {
  const { values } = parseArgs({
    args: argv,
    options: {
      config: { type: "string", short: "c", default: "arc.toml" },
      "dry-run": { type: "boolean", default: false },
    },
  });

  const opts: RunOptions = {
    config: values.config ?? "arc.toml",
    dryRun: values["dry-run"] ?? false,
  };

  const config = await loadConfig(opts.config);
  const result = await execute(config, { dryRun: opts.dryRun });

  if (result.success) {
    console.log("Run completed successfully.");
  } else {
    console.error("Run failed:", result.error);
    process.exitCode = 1;
  }
}
`,
    },
    newFile: {
      name: "src/commands/run.ts",
      contents: `import { parseArgs } from "node:util";
import { loadConfig } from "../config.js";
import { execute } from "../executor.js";
import { createLogger, type Logger } from "../logger.js";

interface RunOptions {
  config: string;
  dryRun: boolean;
  verbose: boolean;
}

export async function run(argv: string[]) {
  const { values } = parseArgs({
    args: argv,
    options: {
      config: { type: "string", short: "c", default: "arc.toml" },
      "dry-run": { type: "boolean", default: false },
      verbose: { type: "boolean", short: "v", default: false },
    },
  });

  const opts: RunOptions = {
    config: values.config ?? "arc.toml",
    dryRun: values["dry-run"] ?? false,
    verbose: values.verbose ?? false,
  };

  const logger: Logger = createLogger({ verbose: opts.verbose });

  const config = await loadConfig(opts.config);
  logger.debug("Loaded config from %s", opts.config);

  const result = await execute(config, { dryRun: opts.dryRun, logger });
  logger.debug("Execution finished in %dms", result.elapsed);

  if (result.success) {
    console.log("Run completed successfully.");
  } else {
    console.error("Run failed:", result.error);
    process.exitCode = 1;
  }
}
`,
    },
  },
  {
    oldFile: {
      name: "src/logger.ts",
      contents: "",
    },
    newFile: {
      name: "src/logger.ts",
      contents: `export interface Logger {
  info(message: string, ...args: unknown[]): void;
  debug(message: string, ...args: unknown[]): void;
  error(message: string, ...args: unknown[]): void;
}

interface LoggerOptions {
  verbose: boolean;
}

export function createLogger({ verbose }: LoggerOptions): Logger {
  return {
    info(message, ...args) {
      console.log(message, ...args);
    },
    debug(message, ...args) {
      if (verbose) {
        console.log("[debug]", message, ...args);
      }
    },
    error(message, ...args) {
      console.error(message, ...args);
    },
  };
}
`,
    },
  },
  {
    oldFile: {
      name: "src/executor.ts",
      contents: `import type { Config } from "./config.js";

interface ExecuteOptions {
  dryRun: boolean;
}

interface ExecuteResult {
  success: boolean;
  error?: string;
}

export async function execute(
  config: Config,
  options: ExecuteOptions,
): Promise<ExecuteResult> {
  if (options.dryRun) {
    console.log("Dry run — skipping execution.");
    return { success: true };
  }

  try {
    for (const step of config.steps) {
      await step.run();
    }
    return { success: true };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { success: false, error: message };
  }
}
`,
    },
    newFile: {
      name: "src/executor.ts",
      contents: `import type { Config } from "./config.js";
import type { Logger } from "./logger.js";

interface ExecuteOptions {
  dryRun: boolean;
  logger: Logger;
}

interface ExecuteResult {
  success: boolean;
  elapsed: number;
  error?: string;
}

export async function execute(
  config: Config,
  options: ExecuteOptions,
): Promise<ExecuteResult> {
  const start = performance.now();

  if (options.dryRun) {
    options.logger.info("Dry run — skipping execution.");
    return { success: true, elapsed: performance.now() - start };
  }

  try {
    for (const step of config.steps) {
      options.logger.debug("Running step: %s", step.name);
      await step.run();
    }
    return { success: true, elapsed: performance.now() - start };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { success: false, elapsed: performance.now() - start, error: message };
  }
}
`,
    },
  },
];

const BLOCK_COUNT = 5;

function DiffStat({ additions, deletions }: { additions: number; deletions: number }) {
  const total = additions + deletions;
  const addBlocks = total === 0 ? 0 : Math.round((additions / total) * BLOCK_COUNT);
  const delBlocks = BLOCK_COUNT - addBlocks;

  return (
    <div className="flex items-center gap-2 font-mono text-xs">
      <span className="font-semibold text-mint">+{additions.toLocaleString()}</span>
      <span className="font-semibold text-coral">-{deletions.toLocaleString()}</span>
      <div className="flex gap-0.5">
        {Array.from({ length: BLOCK_COUNT }, (_, i) => (
          <span
            key={i}
            className={`inline-block size-2.5 rounded-sm ${i < addBlocks ? "bg-mint" : "bg-coral"}`}
          />
        ))}
      </div>
    </div>
  );
}

export default function RunFilesChanged() {
  const [checkpoint, setCheckpoint] = useState(checkpoints[0].id);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <div className="relative">
          <select
            value={checkpoint}
            onChange={(e) => setCheckpoint(e.target.value)}
            className="appearance-none rounded-md border border-white/[0.06] bg-navy-800/80 py-2 pl-3 pr-8 text-sm text-ice-100 outline-none transition-colors focus:border-teal-500/40 focus:ring-0"
          >
            {checkpoints.map((cp) => (
              <option key={cp.id} value={cp.id}>{cp.label}</option>
            ))}
          </select>
          <ChevronDownIcon className="pointer-events-none absolute right-2 top-1/2 size-4 -translate-y-1/2 text-navy-600" />
        </div>
        <div className="ml-auto flex items-center gap-3">
          <DiffStat additions={567} deletions={234} />
          <button
            type="button"
            title="Settings"
            className="flex size-8 items-center justify-center rounded-md border border-white/[0.06] bg-navy-800/80 text-ice-300 transition-colors hover:bg-white/[0.04] hover:text-white"
          >
            <Cog6ToothIcon className="size-4" />
          </button>
        </div>
      </div>

      {files.map(({ oldFile, newFile }) => (
        <div
          key={newFile.name}
          className="rounded-md overflow-hidden border border-white/[0.06]"
        >
          <MultiFileDiff
            oldFile={oldFile}
            newFile={newFile}
            options={{
              diffStyle: "split",
              theme: "pierre-dark",
              lineDiffType: "word",
            }}
          />
        </div>
      ))}
    </div>
  );
}
