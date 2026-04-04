import { Command } from "commander";
import chalk from "chalk";
import { planService } from "../services/planService";
import { logger } from "../utils/logger";
import { PlanRecord } from "../types/plan";

const collectMilestone = (value: string, previous: string[]): string[] => {
  return [...previous, value];
};

function renderPlan(plan: PlanRecord): void {
  logger.info(`${chalk.bold(plan.name)}${plan.description ? ` — ${plan.description}` : ""}`);
  logger.info(`Milestones (${plan.milestones.length}):`);
  plan.milestones.forEach((milestone, index) => {
    logger.info(`  ${index + 1}. ${milestone}`);
  });
  logger.info(`Tasks (${plan.tasks.length}):`);
  plan.tasks.forEach((task, index) => {
    logger.info(`  ${index + 1}. [${task.status}] ${task.title}`);
  });
}

export const registerPlanCommands = (program: Command): void => {
  const plan = program.command("plan").description("Plan management commands");

  plan
    .command("init")
    .description("Initialize a new Dobby plan")
    .requiredOption("-n, --name <name>", "Name of the plan")
    .option("-d, --description <description>", "Description of the plan")
    .option(
      "-m, --milestone <milestone>",
      "Add a milestone (repeat for multiple milestones)",
      collectMilestone,
      [] as string[],
    )
    .action(async (options) => {
      const milestones: string[] = Array.isArray(options.milestone) ? options.milestone : [];
      const planRecord = await planService.initPlan({
        name: options.name,
        description: options.description,
        milestones,
      });

      logger.success("Plan initialized successfully.");
      renderPlan(planRecord);
    });

  plan
    .command("show")
    .description("Show the current plan")
    .action(async () => {
      const planRecord = await planService.getPlan();
      if (!planRecord) {
        logger.warn("No plan found. Use `dobby plan init` to create one.");
        return;
      }
      renderPlan(planRecord);
    });
};
