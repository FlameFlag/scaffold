export async function runCommand(argv, options = {}) {
  const proc = Bun.spawn(argv, {
    stdout: "inherit",
    stderr: "inherit",
    ...options,
  });
  const exitCode = await proc.exited;

  if (exitCode !== 0) {
    throw new Error(`${argv[0]} failed with ${exitCode}`);
  }
}
