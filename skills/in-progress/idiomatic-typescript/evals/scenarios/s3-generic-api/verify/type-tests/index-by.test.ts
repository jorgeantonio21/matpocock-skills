import { indexBy } from "../src/index.js";

type Equal<A, B> =
  (<T>() => T extends A ? 1 : 2) extends
  (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;

type Row = { id: "one" | "two"; value: number };
const rows: readonly Row[] = [];
const result = indexBy(rows, (row) => row.id);
type KeepsKeyType = Expect<Equal<typeof result, Map<"one" | "two", Row>>>;

// @ts-expect-error: objects are not property keys
indexBy(rows, (row) => ({ id: row.id }));

const proof: KeepsKeyType = true;
void proof;
