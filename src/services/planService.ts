import { randomUUID } from "crypto";
import { PlanInitInput, PlanRecord } from "../types/plan";
import { storage } from "./storage";

class PlanService {
  async initPlan(input: PlanInitInput): Promise<PlanRecord> {
    const state = await storage.readState();
    if (state.plan) {
      throw new Error("A plan already exists. Use `dobby plan show` or reset the state.");
    }

    const now = new Date().toISOString();
    const plan: PlanRecord = {
      id: randomUUID(),
      name: input.name,
      description: input.description,
      milestones: input.milestones,
      tasks: [],
      createdAt: now,
      updatedAt: now,
    };

    await storage.writeState({ plan });
    return plan;
  }

  async getPlan(): Promise<PlanRecord | null> {
    const state = await storage.readState();
    return state.plan;
  }

  async updatePlan(update: Partial<PlanRecord>): Promise<PlanRecord> {
    const state = await storage.readState();
    if (!state.plan) {
      throw new Error("No plan found. Initialize one with `dobby plan init`.");
    }

    const updatedPlan: PlanRecord = {
      ...state.plan,
      ...update,
      updatedAt: new Date().toISOString(),
    };
    await storage.writeState({ plan: updatedPlan });
    return updatedPlan;
  }
}

export const planService = new PlanService();
