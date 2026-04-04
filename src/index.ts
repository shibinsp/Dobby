#!/usr/bin/env node
import { Command } from "commander";
import packageJson from "../package.json";
import { registerPlanCommands } from "./commands/plan";
import { registerTaskCommands } from "./commands/task";
import { forgeRunner } from "./services/forgeRunner";
import { logger } from "./utils/logger";

const program = new Command();
const dobbyCommands = new Set(["plan", "task"]);

async function delegateToForgeIfNeeded(argv: string[]): Promise<number | null> {
  const args = argv.slice(2);
  const firstCommand = args.find((arg) => !arg.startsWith("-"));
  const shouldDelegate = args.length === 0 || !firstCommand || !dobbyCommands.has(firstCommand);
  if (shouldDelegate) {
    return forgeRunner.run(args);
  }
  return null;
}

program
  .name("dobby")
  .description("Dobby CLI — automation companion")
  .version(packageJson.version);

registerPlanCommands(program);
registerTaskCommands(program);

export async function run(argv = process.argv): Promise<void> {
  try {
    const delegatedCode = await delegateToForgeIfNeeded(argv);
    if (delegatedCode !== null) {
      process.exit(delegatedCode);
      return;
    }
    await program.parseAsync(argv);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    logger.error(message);
    process.exit(1);
  }
}

run();
