import { parseApiUser, type ApiUser } from "../../dist/index.js";

const user: ApiUser = parseApiUser({ name: "Ada", loginCount: 0, enabled: true });
void user;
