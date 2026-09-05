import {
  indexBy,
  parseApiUser,
  parseUserId,
  type ApiUser,
  type UserId,
} from "../src/index.js";

type Equal<A, B> =
  (<T>() => T extends A ? 1 : 2) extends
  (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;

const user: ApiUser = parseApiUser({ name: "Ada", loginCount: 0, enabled: false });
const userId: UserId = parseUserId("usr_ada");
const byId = indexBy([user], (value) => value.name);
const filtered = [1, undefined, 0].filter((value) => value !== undefined);

type UserMapKeyIsString = Expect<Equal<typeof byId, Map<string, ApiUser>>>;
type FilteredIsNumberArray = Expect<Equal<typeof filtered, number[]>>;

void userId;
const typeTests: [UserMapKeyIsString, FilteredIsNumberArray] = [true, true];
void typeTests;
