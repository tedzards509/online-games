import type { GameKind, Membership, Schemas } from './types';

export async function request<T>(
  path: string,
  body?: unknown,
  token?: string,
): Promise<T> {
  const response = await fetch(`/api${path}`, {
    method: body === undefined ? 'GET' : 'POST',
    headers: {
      ...(body === undefined ? {} : { 'Content-Type': 'application/json' }),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) {
    const data = await response.json().catch(() => null);
    throw new Error(data?.error ?? `Request failed (${response.status})`);
  }
  return response.json();
}

export function createRoom(
  nickname: string,
  game: GameKind,
  config: Schemas['Config'],
): Promise<Membership> {
  return request('/rooms', { nickname, game, config });
}

export function joinRoom(code: string, nickname: string): Promise<Membership> {
  return request(`/rooms/${encodeURIComponent(code)}/join`, { nickname });
}

export function saveMembership(member: Membership): void {
  localStorage.setItem(`arena:${member.code}`, JSON.stringify(member));
}

export function loadMembership(code: string): Membership | null {
  try {
    const member = JSON.parse(localStorage.getItem(`arena:${code}`) ?? 'null');
    return member?.code === code &&
      typeof member.token === 'string' &&
      Number.isInteger(member.player)
      ? member
      : null;
  } catch {
    return null;
  }
}

export function savedRooms(): string[] {
  return Object.keys(localStorage)
    .filter((key) => key.startsWith('arena:'))
    .map((key) => key.slice(6))
    .filter((code) => loadMembership(code) !== null);
}

export async function downloadReplay(
  code: string,
  id: string,
  token: string,
): Promise<void> {
  const replay = await request<unknown>(
    `/rooms/${code}/replays/${id}`,
    undefined,
    token,
  );
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(replay, null, 2)], { type: 'application/json' }),
  );
  const link = document.createElement('a');
  link.href = url;
  link.download = `${code}-${id}.json`;
  link.click();
  URL.revokeObjectURL(url);
}
