export interface User {
  name: string;
  retries: number;
  enabled: boolean;
}

export function parseUser(input: unknown): User {
  const value = input as User;
  return {
    name: value.name || "Anonymous",
    retries: value.retries || 3,
    enabled: value.enabled || true,
  };
}
