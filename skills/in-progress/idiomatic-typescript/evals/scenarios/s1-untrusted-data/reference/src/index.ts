export interface User {
  name: string;
  retries: number;
  enabled: boolean;
}

function isRecord(input: unknown): input is Record<string, unknown> {
  return typeof input === "object" && input !== null && !Array.isArray(input);
}

export function parseUser(input: unknown): User {
  if (!isRecord(input)) {
    throw new TypeError("user must be an object");
  }
  if (typeof input.name !== "string") {
    throw new TypeError("user.name must be a string");
  }
  if (typeof input.retries !== "number" || !Number.isInteger(input.retries)) {
    throw new TypeError("user.retries must be an integer");
  }
  if (typeof input.enabled !== "boolean") {
    throw new TypeError("user.enabled must be a boolean");
  }
  return { name: input.name, retries: input.retries, enabled: input.enabled };
}
