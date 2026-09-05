export interface ApiUser {
  name: string;
  loginCount: number;
  enabled: boolean;
}

export function isRecord(input: unknown): input is Record<string, unknown> {
  return typeof input === "object" && input !== null && !Array.isArray(input);
}

export function parseApiUser(input: unknown): ApiUser {
  if (!isRecord(input)) {
    throw new TypeError("user must be an object");
  }
  if (typeof input.name !== "string") {
    throw new TypeError("user.name must be a string");
  }
  if (typeof input.loginCount !== "number" || !Number.isInteger(input.loginCount)) {
    throw new TypeError("user.loginCount must be an integer");
  }
  if (typeof input.enabled !== "boolean") {
    throw new TypeError("user.enabled must be a boolean");
  }
  return {
    name: input.name,
    loginCount: input.loginCount,
    enabled: input.enabled,
  };
}
