import fs from "fs-extra";
import os from "os";
import path from "path";
import { DobbyState } from "../types/plan";

const DATA_DIR = path.join(os.homedir(), ".dobby-cli");
const DATA_FILE = path.join(DATA_DIR, "state.json");

const defaultState: DobbyState = {
  plan: null,
};

export class StorageService {
  async ensureDataFile(): Promise<void> {
    await fs.ensureDir(DATA_DIR);
    if (!(await fs.pathExists(DATA_FILE))) {
      await fs.writeJson(DATA_FILE, defaultState, { spaces: 2 });
    }
  }

  async readState(): Promise<DobbyState> {
    await this.ensureDataFile();
    const content = await fs.readJson(DATA_FILE);
    return {
      ...defaultState,
      ...content,
    } satisfies DobbyState;
  }

  async writeState(next: DobbyState): Promise<void> {
    await this.ensureDataFile();
    await fs.writeJson(DATA_FILE, next, { spaces: 2 });
  }

  async reset(): Promise<void> {
    await fs.remove(DATA_FILE);
  }
}

export const storage = new StorageService();
