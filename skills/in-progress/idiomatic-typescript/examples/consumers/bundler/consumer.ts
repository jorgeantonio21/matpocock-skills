import { indexBy } from "../../dist/index.js";

const values = indexBy([{ id: "one" }], (value) => value.id);
void values;
