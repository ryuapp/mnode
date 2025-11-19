const __internal = globalThis[Symbol.for("mdeno.internal")];

const env = JSON.parse(__internal.getEnv());
const argv = JSON.parse(__internal.getArgv());

function exit(code = 0) {
  internal.exit(code);
}

export { argv, env, exit };

export default { env, argv, exit };
