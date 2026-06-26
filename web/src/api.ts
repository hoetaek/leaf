export async function responseErrorMessage(response: Response): Promise<string> {
  try {
    const body: unknown = await response.clone().json();
    if (body && typeof body === "object" && "error" in body) {
      const error = (body as { error: unknown }).error;
      if (typeof error === "string" && error.trim()) return error;
    }
  } catch {
    // Keep the stable HTTP fallback for non-JSON error bodies.
  }
  return `HTTP ${response.status}`;
}

export function fetchJson<T>(path: string): Promise<T> {
  return fetch(path).then(async (response) => {
    if (!response.ok) throw new Error(await responseErrorMessage(response));
    return response.json() as Promise<T>;
  });
}
