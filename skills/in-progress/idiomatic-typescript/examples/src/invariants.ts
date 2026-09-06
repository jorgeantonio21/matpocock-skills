export type UserId = string & { readonly __brand: "UserId" };

export function parseUserId(input: unknown): UserId {
  if (typeof input !== "string" || !/^usr_[a-z0-9]+$/.test(input)) {
    throw new TypeError("user id must match usr_[a-z0-9]+");
  }
  return input as UserId;
}

export type NonEmptyString = string & { readonly __brand: "NonEmptyString" };

export function isNonEmptyString(value: unknown): value is NonEmptyString {
  return typeof value === "string" && value.length > 0;
}

export function observeReadonlyAlias(): [number, number] {
  const mutable = { count: 1 };
  const view: Readonly<{ count: number }> = mutable;
  const before = view.count;
  mutable.count = 2;
  return [before, view.count];
}
