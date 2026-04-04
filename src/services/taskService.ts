import { randomUUID } from "crypto";
import { planService } from "./planService";
import { TaskInput, TaskRecord, TaskStatus } from "../types/plan";

class TaskService {
  async addTask(input: TaskInput): Promise<TaskRecord> {
    const plan = await planService.getPlan();
    if (!plan) {
      throw new Error("No plan found. Create one with `dobby plan init` before adding tasks.");
    }

    const now = new Date().toISOString();
    const task: TaskRecord = {
      id: randomUUID(),
      title: input.title,
      status: "pending",
      notes: input.notes,
      createdAt: now,
      updatedAt: now,
    };

    const nextTasks = [...plan.tasks, task];
    await planService.updatePlan({ tasks: nextTasks });
    return task;
  }

  async listTasks(): Promise<TaskRecord[]> {
    const plan = await planService.getPlan();
    if (!plan) {
      throw new Error("No plan found. Create one with `dobby plan init`.");
    }
    return plan.tasks;
  }

  async updateStatus(taskId: string, status: TaskStatus): Promise<TaskRecord> {
    const plan = await planService.getPlan();
    if (!plan) {
      throw new Error("No plan found. Create one with `dobby plan init`.");
    }

    const tasks = plan.tasks.map((task) => {
      if (task.id !== taskId) {
        return task;
      }
      return {
        ...task,
        status,
        updatedAt: new Date().toISOString(),
      } satisfies TaskRecord;
    });

    const updatedTask = tasks.find((task) => task.id === taskId);
    if (!updatedTask) {
      throw new Error(`Task with id ${taskId} not found.`);
    }

    await planService.updatePlan({ tasks });
    return updatedTask;
  }
}

export const taskService = new TaskService();
