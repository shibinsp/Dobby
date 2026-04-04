import chalk from "chalk";
import { TaskRecord } from "../types/plan";

const prefix = chalk.gray("[dobby]");

const statusColorMap = {
  completed: chalk.green,
  in_progress: chalk.yellow,
  pending: chalk.cyan,
} as const;

export interface FormatTaskOptions {
  index?: number;
  includeId?: boolean;
}

export const formatTask = (task: TaskRecord, options: FormatTaskOptions = {}): string => {
  const prefixLabel = typeof options.index === "number" ? `${options.index + 1}. ` : "- ";
  const colorFn = statusColorMap[task.status];
  const notesText = task.notes ? chalk.dim(` — ${task.notes}`) : "";
  const idText = options.includeId ? ` ${chalk.gray(`#${task.id.slice(0, 8)}`)}` : "";
  return `${prefixLabel}${colorFn(`[${task.status}]`)} ${chalk.bold(task.title)}${notesText}${idText}`;
};

export const logger = {
  info(message: string) {
    console.log(prefix, chalk.white(message));
  },
  success(message: string) {
    console.log(prefix, chalk.green(message));
  },
  warn(message: string) {
    console.warn(prefix, chalk.yellow(message));
  },
  error(message: string) {
    console.error(prefix, chalk.red(message));
  },
};
