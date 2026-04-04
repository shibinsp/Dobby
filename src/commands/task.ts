import { Command } from "commander";
import chalk from "chalk";
import { taskService } from "../services/taskService";
import { logger } from "../utils/logger";
import { TaskRecord, TaskStatus } from "../types/plan";

const statusChoices: TaskStatus[] = ["pending", "in_progress", "completed"];

const formatTask = (task: TaskRecord, index?: number): string => {
  const prefix = typeof index === "number" ? `${index + 1}. ` : "- ";
  const statusColor =
    task.status === "completed" ? chalk.green : task.status === "in_progress" ? chalk.yellow : chalk.cyan;
  return `${prefix}${statusColor(`[${task.status}]`)} ${chalk.bold(task.title)}${task.notes ? ` — ${task.notes}` : ""}`;
};

export const registerTaskCommands = (program: Command): void => {
  const task = program.command("task").description("Task management commands");

  task
    .command("add")
    .description("Add a task to the current plan")
    .argument("<title>", "Title of the task")
    .option("-n, --notes <notes>", "Optional notes")
    .action(async (title, options) => {
      const record = await taskService.addTask({ title, notes: options.notes });
      logger.success(`Task created (${record.id}).`);
      logger.info(formatTask(record));
    });

  task
    .command("list")
    .description("List plan tasks")
    .option("-s, --status <status>", `Filter by status: ${statusChoices.join(", ")}`)
    .action(async (options) => {
      const tasks = await taskService.listTasks();
      const filtered = options.status
        ? tasks.filter((taskRecord) => taskRecord.status === options.status)
        : tasks;
      if (!filtered.length) {
        logger.warn("No tasks matched your criteria.");
        return;
      }
      filtered.forEach((taskRecord, index) => logger.info(formatTask(taskRecord, index)));
    });

  task
    .command("status")
    .description("Update task status")
    .argument("<id>", "Task identifier")
    .argument("<status>", `New status (${statusChoices.join(", ")})`)
    .action(async (taskId, status) => {
      if (!statusChoices.includes(status)) {
        throw new Error(`Invalid status. Must be one of: ${statusChoices.join(", ")}`);
      }
      const updated = await taskService.updateStatus(taskId, status);
      logger.success("Task updated.");
      logger.info(formatTask(updated));
    });
};
