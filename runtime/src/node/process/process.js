const internal = globalThis[Symbol.for("mnode.internal")];

const env = JSON.parse(internal.getEnv());
const argv = JSON.parse(internal.getArgv());

function exit(code = 0) {
  internal.exit(code);
}

export { argv, env, exit };

export default { env, argv, exit };
