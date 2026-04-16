import { spawn, ChildProcess } from 'child_process';
import { execSync } from 'child_process';
import path from 'path';

let appProcess: ChildProcess | null = null;

export async function launchApp(): Promise<void> {
  // Build the app first
  const projectRoot = path.resolve(__dirname, '../..');
  execSync('cargo build --manifest-path src-tauri/Cargo.toml', {
    cwd: projectRoot,
    stdio: 'inherit'
  });

  // Get the binary path
  const isLinux = process.platform === 'linux';
  const isMac = process.platform === 'darwin';
  const isWindows = process.platform === 'win32';

  let binaryPath: string;
  if (isLinux) {
    binaryPath = path.join(projectRoot, 'target/debug/artifex');
  } else if (isMac) {
    binaryPath = path.join(projectRoot, 'target/debug/Artifex');
  } else {
    binaryPath = path.join(projectRoot, 'target/debug/Artifex.exe');
  }

  appProcess = spawn(binaryPath, [], {
    detached: true,
    stdio: 'ignore'
  });

  // Wait for app to start
  await new Promise(resolve => setTimeout(resolve, 3000));
}

export async function closeApp(): Promise<void> {
  if (appProcess) {
    if (process.platform === 'win32') {
      execSync(`taskkill /pid ${appProcess.pid} /T /F`);
    } else {
      process.kill(-appProcess.pid!);
    }
    appProcess = null;
  }
}
