import { invoke } from '@tauri-apps/api/core';

export interface SessionDuration {
  date: string;
  duration: number;
}

export function getSessions(): Promise<SessionDuration[]> {
  return invoke<SessionDuration[]>('get_sessions');
}
