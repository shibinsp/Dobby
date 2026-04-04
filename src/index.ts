#!/usr/bin/env node
import { Command } from "commander";
import packageJson from "../package.json";
import { registerPlanCommands } from "./commands/plan";
import { registerTaskCommands } from "./commands/task";
import { logger } from "./utils/logger";

const program = new Command();

program
  .name("dobby")
  .description("Dobby CLI — automation companion")
  .version(packageJson.version);

registerPlanCommands(program);
registerTaskCommands(program);

export async function run(argv = process.argv): Promise<void> {
  try {
    await program.parseAsync(argv);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    logger.error(message);
    process.exit(1);
  }
}

run();
