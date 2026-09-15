// GitHub Releases-based update check. There's no signed updater manifest
// (no keypair, no latest.json — see .github/workflows/release.yml), so this
// just points the user at the release page rather than auto-installing.
const RELEASES_API = "https://api.github.com/repos/RaiN5417/AssetPile/releases/latest";
const RELEASES_FALLBACK_URL = "https://github.com/RaiN5417/AssetPile/releases/latest";

export interface UpdateInfo {
  version: string;
  url: string;
}

function isNewer(latest: string, current: string): boolean {
  const a = latest.split(".").map(Number);
  const b = current.split(".").map(Number);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}

/** Returns the newer release if one exists, or null if already up to date. */
export async function checkForUpdate(currentVersion: string): Promise<UpdateInfo | null> {
  const res = await fetch(RELEASES_API);
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  const data = (await res.json()) as { tag_name?: string; html_url?: string };
  const latest = (data.tag_name ?? "").replace(/^v/, "");
  if (!latest || !isNewer(latest, currentVersion)) return null;
  return { version: latest, url: data.html_url ?? RELEASES_FALLBACK_URL };
}
