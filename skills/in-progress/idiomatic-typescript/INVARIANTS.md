# Honest TypeScript invariants

Read this when choosing or reviewing a type that claims validation, nominal identity, immutability, authorization, or compatibility. Start with the mistake the type should prevent, then name what establishes the fact.

TypeScript is structurally typed and intentionally permits some unsound behavior for JavaScript compatibility. Static types are erased. A useful type can still prevent many checked-call mistakes, but it must not claim runtime proof it never obtained.

## The guarantee ladder

1. **Structural shape:** checked code can use the named fields and methods. Compatibility is based mainly on members, not declarations.
2. **Intrinsic value property:** a fact about one value, such as a string matching an identifier syntax. A runtime parser establishes it; a brand can record it.
3. **Aggregate property:** a fact about values together, such as sortedness or uniqueness. Every mutation route or an owning abstraction must preserve it.
4. **Contextual admission:** a policy accepted a value at one time, such as authorization or feature eligibility. Recheck at the authoritative operation when policy can change.
5. **Persistent interpretation:** stored bytes or JSON retain their historical meaning across versions. A versioned decoder and migration tests establish this.

For each claim, list construction, deserialization, assertion, mutation, alias, and migration routes. The guarantee is only as strong as the weakest route that reaches the value.

## A primitive is enough

Keep `string` for free text and `number` for ordinary quantities when the wrapper prevents no realistic mix-up and records no checked property. Put two same-typed roles in an object with named fields when names already prevent the mistake. Complexity that closes no path is ceremony.

## Alternatives belong in a union

A discriminated union protects relationships between a state and its data. Keep the discriminant literal and handle local unions exhaustively. At an external decoder, define what happens to a future unknown discriminant instead of assuming the server shares the local closed world.

```ts
export function assertNever(value: never): never {
  throw new Error(`unexpected state: ${JSON.stringify(value)}`);
}
```

A runtime fallback for unknown wire versions can be correct. A broad default over a local union can hide a case the program already knows.

## Parse, then carry the checked representation

A boundary parser accepts `unknown`, checks the properties needed by the operation, and returns the representation the core consumes. A schema may be the parser. If it transforms values, distinguish the schema's input and output types.

Validation proves the checks it performed at that moment. It does not prove authorization, freshness, deep immutability, or future wire compatibility. Keep an inaccurate dependency declaration behind an adapter with the same shape: unsafe input in, checked project type out.

## Assertions carry visible evidence

An assertion is suitable when runtime or control-flow evidence exists but TypeScript cannot express the relationship. Keep that evidence close enough to audit. `JSON.parse(text) as User`, `fetchJson<User>()`, and a helper that returns `value as T` have no such evidence.

An assertion can also state an external contract, such as a framework callback guarantee. Preserve that contract and test its integration. Replacing a visible assertion with an unchecked generic helper makes the same claim harder to find.

## Predicates are two-sided contracts

A declaration `value is T` tells the checker what both branches mean. Returning `false` means the value is not a `T`, so a test for a subset cannot claim the whole wider type.

```ts
export type NonEmptyString = string & { readonly __brand: "NonEmptyString" };

export function isNonEmptyString(value: unknown): value is NonEmptyString {
  return typeof value === "string" && value.length > 0;
}
```

The refined type makes the subset explicit, so the false branch excludes only non-empty strings. Test true and false examples. Prefer ordinary narrowing when it communicates the same fact without a handwritten contract.

## Brands record, parsers establish

A brand separates structurally identical roles for checked callers or records a property a parser established. Keep construction private to the module when practical.

```ts
export type UserId = string & { readonly __brand: "UserId" };

export function parseUserId(input: unknown): UserId {
  if (typeof input !== "string" || !/^usr_[a-z0-9]+$/.test(input)) {
    throw new TypeError("user id must match usr_[a-z0-9]+");
  }
  return input as UserId;
}
```

The assertion is justified by the immediately preceding runtime check. The brand has no runtime representation, and `any` or another assertion can bypass it. It is not a security boundary. Recheck current authorization where the protected operation occurs.

## Readonly is a permission, not ownership

A readonly view blocks writes through that view. It does not freeze the object, recurse through nested values, or stop a writable alias.

```ts
export function observeReadonlyAlias(): [number, number] {
  const mutable = { count: 1 };
  const view: Readonly<{ count: number }> = mutable;
  const before = view.count;
  mutable.count = 2;
  return [before, view.count];
}
```

When a stable snapshot matters, copy at the boundary, encapsulate mutation behind one owner, or use runtime immutability appropriate to the workload. Test the promised stability. Deep mapped types still provide static permissions, not ownership.

## Aggregate and persistent properties need route audits

A sorted collection stays sorted only if every insertion, replacement, and deserialization route preserves order. Prefer an owning interface with a small mutation surface when callers should rely on the property. A type alias alone cannot police writes through aliases.

When a stored format gains stricter validation, version the interpretation. Test historical valid data, new valid data, and corrupt data. A migration must preserve the old meaning rather than route an old value through a new default. Compatibility is observable behavior, not merely assignability between two interfaces.
