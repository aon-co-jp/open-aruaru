export type AiProvider = {
  id: string;
  name: string;
  vendor: string;
  cost_tier: string;
  strengths: string[];
  speech_quality?: string;
  translation_quality?: string;
  score?: number;
  reason?: string;
};

const DEV_KEY = "aruaru_ai_development";
const ENG_KEY = "aruaru_ai_english";

export function getSelectedAi(mode: "development" | "english_conversation"): string | null {
  const k = mode === "development" ? DEV_KEY : ENG_KEY;
  return localStorage.getItem(k);
}

export function setSelectedAi(mode: "development" | "english_conversation", id: string): void {
  const k = mode === "development" ? DEV_KEY : ENG_KEY;
  localStorage.setItem(k, id);
}

export function renderAiCard(p: AiProvider, selected: boolean, mode: string): string {
  const badges = [
    `<span class="tag cost-${p.cost_tier}">${p.cost_tier}</span>`,
    `<span class="tag">${p.vendor}</span>`,
  ];
  if (p.speech_quality) badges.push(`<span class="tag">音声:${p.speech_quality}</span>`);
  if (p.translation_quality) badges.push(`<span class="tag">翻訳:${p.translation_quality}</span>`);
  const score = p.score != null ? `<span class="score">${p.score}pt</span>` : "";
  return `
<div class="ai-card ${selected ? "selected" : ""}" data-id="${p.id}">
  <div class="ai-card-head">
    <strong>${p.name}</strong> ${score}
    <div class="tags">${badges.join(" ")}</div>
  </div>
  <p class="muted">${p.strengths.join(" · ")}</p>
  ${p.reason ? `<p class="reason">${p.reason}</p>` : ""}
  <button type="button" class="primary select-ai" data-id="${p.id}" data-mode="${mode}">
    ${selected ? "選択中" : "この AI を選択"}
  </button>
</div>`;
}

export async function fetchRecommend(
  api: <T>(path: string, init?: RequestInit) => Promise<T>,
  useCase: string
): Promise<AiProvider[]> {
  const res = await api<{ ok?: boolean; recommended?: AiProvider[] }>(
    `/ai/recommend?use_case=${encodeURIComponent(useCase)}`
  );
  return res.recommended ?? [];
}

export function wireAiSelect(container: HTMLElement, mode: "development" | "english_conversation"): void {
  const selected = getSelectedAi(mode);
  container.querySelectorAll(".ai-card").forEach((card) => {
    const id = (card as HTMLElement).dataset.id;
    if (id === selected) card.classList.add("selected");
  });
  container.querySelectorAll(".select-ai").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = (btn as HTMLButtonElement).dataset.id!;
      setSelectedAi(mode, id);
      container.querySelectorAll(".ai-card").forEach((c) => c.classList.remove("selected"));
      container.querySelectorAll(".select-ai").forEach((b) => {
        (b as HTMLButtonElement).textContent = "この AI を選択";
      });
      const card = btn.closest(".ai-card");
      card?.classList.add("selected");
      (btn as HTMLButtonElement).textContent = "選択中";
    });
  });
}
