const BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8001";

class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    public message: string,
    public details?: unknown
  ) {
    super(message);
    this.name = "ApiError";
  }
}

type RequestOptions = {
  token?: string | null;
  params?: Record<string, string | number | boolean | undefined>;
};

let cachedToken: string | null = null;

async function getAuthToken(): Promise<string | null> {
  if (cachedToken) return cachedToken;
  try {
    const res = await fetch(`${BASE}/api/v1/auth/session`);
    if (res.ok) {
      const data = await res.json();
      cachedToken = data.access_token || null;
      return cachedToken;
    }
  } catch {
    // If backend is unreachable or building statically
  }
  return null;
}

export function setExplicitToken(token: string | null) {
  cachedToken = token;
}

async function request<T>(method: string, path: string, body?: unknown, opts: RequestOptions = {}): Promise<T> {
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  const fullPath = normalizedPath.startsWith("/api/v1") || normalizedPath.startsWith("/health") || normalizedPath.startsWith("/metrics")
    ? normalizedPath
    : `/api/v1${normalizedPath}`;

  const url = new URL(`${BASE}${fullPath}`);

  if (opts.params) {
    for (const [k, v] of Object.entries(opts.params)) {
      if (v !== undefined) url.searchParams.set(k, String(v));
    }
  }

  const headers: Record<string, string> = {
    Accept: "application/json",
  };
  if (body) headers["Content-Type"] = "application/json";

  const token = opts.token !== undefined ? opts.token : await getAuthToken();
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const res = await fetch(url.toString(), {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
    cache: "no-store",
  });

  if (res.status === 204) return undefined as T;

  const data = await res.json().catch(() => null);

  if (!res.ok) {
    const error = data?.error || {};
    throw new ApiError(
      res.status,
      error.code || "ERR_HTTP",
      typeof error === "string" ? error : error.message || res.statusText,
      error.details
    );
  }

  return data;
}

export const api = {
  get: <T>(path: string, opts?: RequestOptions) => request<T>("GET", path, undefined, opts),
  post: <T>(path: string, body?: unknown, opts?: RequestOptions) => request<T>("POST", path, body, opts),
  patch: <T>(path: string, body?: unknown, opts?: RequestOptions) => request<T>("PATCH", path, body, opts),
  delete: <T>(path: string, opts?: RequestOptions) => request<T>("DELETE", path, undefined, opts),
};

export { ApiError };
