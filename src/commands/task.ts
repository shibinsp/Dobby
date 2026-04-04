import { Command } from "commander";
import { taskService } from "../services/taskService";
import { formatTask, logger } from "../utils/logger";
import { TASK_STATUS_VALUES, TaskRecord, TaskStatus } from "../types/plan";

const isNumeric = (value: string): boolean => /^\d+$/.test(value.trim());

const resolveTaskFromIdentifier = (identifier: string, tasks: TaskRecord[]): TaskRecord => {
  const trimmed = identifier.trim();
  if (isNumeric(trimmed)) {
    const index = Number(trimmed) - 1;
    if (index < 0 || index >= tasks.length) {
      throw new Error(`Task index ${trimmed} is out of range. There are ${tasks.length} task(s).`);
    }
    return tasks[index];
  }

  const matches = tasks.filter((task) => task.id === trimmed || task.id.startsWith(trimmed));
  if (matches.length === 0) {
    throw new Error(`No task matched "${trimmed}". Run \`dobby task list\` to inspect IDs.`);
  }
  if (matches.length > 1) {
    const hints = matches.map((task) => `${task.title} (#${task.id.slice(0, 8)})`).join(", ");
    throw new Error(`Identifier "${trimmed}" is ambiguous: ${hints}`);
  }
  return matches[0];
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
      logger.info(formatTask(record, { includeId: true }));
    });

  task
    .command("list")
    .description("List plan tasks")
    .option("-s, --status <status>", `Filter by status: ${TASK_STATUS_VALUES.join(", ")}`)
    .action(async (options) => {
      const requestedStatus =
        typeof options.status === "string" ? (options.status.toLowerCase() as TaskStatus) : undefined;
      if (requestedStatus && !TASK_STATUS_VALUES.includes(requestedStatus)) {
        throw new Error(`Invalid status. Use one of: ${TASK_STATUS_VALUES.join(", ")}`);
      }

      const tasks = await taskService.listTasks();
      const filtered = requestedStatus ? tasks.filter((taskRecord) => taskRecord.status === requestedStatus) : tasks;
      if (!filtered.length) {
        logger.warn("No tasks matched your criteria.");
        return;
      }
      filtered.forEach((taskRecord, index) => logger.info(formatTask(taskRecord, { index, includeId: true })));
    });

  task
    .command("status")
    .description("Update task status")
    .argument("<task>", "Task ID prefix or 1-based index")
    .argument("<status>", `New status (${TASK_STATUS_VALUES.join(", ")})`)
    .action(async (taskIdentifier, status) => {
      const nextStatus = status.toLowerCase() as TaskStatus;
      if (!TASK_STATUS_VALUES.includes(nextStatus)) {
        throw new Error(`Invalid status. Must be one of: ${TASK_STATUS_VALUES.join(", ")}`);
      }

      const tasks = await taskService.listTasks();
      const targetTask = resolveTaskFromIdentifier(taskIdentifier, tasks);
      const updated = await taskService.updateStatus(targetTask.id, nextStatus);
      logger.success("Task updated.");
      logger.info(formatTask(updated, { includeId: true }));
    });
};
