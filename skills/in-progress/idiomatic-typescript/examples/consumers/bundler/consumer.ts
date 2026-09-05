import { indexBy } from "../../src/index";

const values = indexBy([{ id: "one" }], (value) => value.id);
void values;
