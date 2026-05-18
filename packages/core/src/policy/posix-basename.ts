export function posixBasename(value: string): string {
  const normalized = value.replace(/\\/gu, "/").replace(/\/+$/u, "");
  if (!normalized) {
    return "";
  }
  const separator = normalized.lastIndexOf("/");
  return separator === -1 ? normalized : normalized.slice(separator + 1);
}
