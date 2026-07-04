export async function gql(query: string, variables?: Record<string, unknown>) {
  const endpoint = import.meta.env.VITE_ARUARU_GRAPHQL ?? "http://127.0.0.1:8787/graphql";
  const res = await fetch(endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query, variables })
  });
  if (!res.ok) throw new Error(`GraphQL transport failed: ${res.status}`);
  const json = await res.json();
  if (json.errors?.length) throw new Error(json.errors.map((e: { message: string }) => e.message).join("; "));
  return json.data;
}
