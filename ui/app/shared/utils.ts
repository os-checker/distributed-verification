import { ofetch } from "ofetch";

// Download JSON
export function download<T>(url: string): Promise<T> {
  return ofetch<T>(url, { parseResponse: JSON.parse });
}
