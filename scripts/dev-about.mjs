import { spawn } from "node:child_process";

const child = spawn("tauri", ["dev"], {
  env: {
    ...process.env,
    BROWSER_SWITCHER_SHOW_ABOUT_ON_START: "1",
  },
  shell: true,
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 0);
});
