import { greet } from "../src/index.js";

const greeting: string = greet("Ada");
// @ts-expect-error: the public API accepts a name string
void greet(42);
void greeting;
