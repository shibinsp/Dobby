import fs from "fs-extra";
import path from "path";
import os from "os";
import { spawn } from "child_process";
import { logger } from "../utils/logger";

class ForgeRunner {
  private readonly forgeRoot = path.resolve(__dirname, "..", "..", "vendor", "forgecode");
  private readonly binaryName = os.platform() === "win32" ? "forge.exe" : "forge";
  private readonly binaryPath = path.join(this.forgeRoot, "target", "release", this.binaryName);
  private buildPromise: Promise<void> | null = null;

  private async ensureForgeSource(): Promise<void> {
    const exists = await fs.pathExists(this.forgeRoot);
    if (!exists) {
      throw new Error(
        "Forge submodule missing. Run `git submodule update --init --recursive` before invoking delegated commands.",
      );
    }
  }

  private async runProcess(command: string, args: string[], cwd: string): Promise<void> {
    await new Promise<void>((resolve, reject) => {
      const child = spawn(command, args, {
        cwd,
        stdio: "inherit",
      });
      child.on("error", (error) => {
        if ((error as NodeJS.ErrnoException).code === "ENOENT") {
          reject(new Error(`Missing dependency: ${command} is required to delegate to Forge.`));
          return;
        }
        reject(error);
      });
      child.on("close", (code) => {
        if (code && code !== 0) {
          reject(new Error(`${command} exited with code ${code}`));
          return;
        }
        resolve();
      });
    });
  }

  private async buildBinary(): Promise<void> {
    await this.ensureForgeSource();
    if (await fs.pathExists(this.binaryPath)) {
      return;
    }
    logger.info("Building Forge CLI from vendored sources. This may take a moment...");
    await this.runProcess("cargo", ["build", "--release"], this.forgeRoot);
  }

  private async ensureBinary(): Promise<void> {
    if (!this.buildPromise) {
      this.buildPromise = this.buildBinary();
    }
    await this.buildPromise;
  }

  async run(args: string[]): Promise<number> {
    await this.ensureBinary();
    return await new Promise<number>((resolve, reject) => {
      const child = spawn(this.binaryPath, args, {
        cwd: process.cwd(),
        stdio: "inherit",
      });
      child.on("error", reject);
      child.on("close", (code) => resolve(code ?? 0));
    });
  }
}

export const forgeRunner = new ForgeRunner();
