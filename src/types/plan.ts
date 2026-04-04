export type TaskStatus = "pending" | "in_progress" | "completed";
export const TASK_STATUS_VALUES: TaskStatus[] = ["pending", "in_progress", "completed"];

export interface TaskRecord {
  id: string;
  title: string;
  status: TaskStatus;
  notes?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PlanRecord {
  id: string;
  name: string;
  description?: string;
  milestones: string[];
  tasks: TaskRecord[];
  createdAt: string;
  updatedAt: string;
}

export interface DobbyState {
  plan: PlanRecord | null;
}

export interface PlanInitInput {
  name: string;
  description?: string;
  milestones: string[];
}

export interface TaskInput {
  title: string;
  notes?: string;
}
