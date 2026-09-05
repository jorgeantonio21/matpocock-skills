type UserId = string & { readonly __brand: "UserId" };
type JobId = string & { readonly __brand: "JobId" };
declare function loadUser(id: UserId): void;
declare const jobId: JobId;
loadUser(jobId);
