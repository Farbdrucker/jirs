import { get } from 'svelte/store';
import { auth } from './auth';
import { goto } from '$app/navigation';
import type {
  AuthResponse, Board, Comment, Project, ProjectMember, RepoLink, Sprint, Tag,
  Ticket, TicketDetail, TicketSummary, TicketLink, User, UserSummary, ActivityEntry
} from './types';

const BASE = '/api';

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const authState = get(auth);
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {})
  };
  if (authState.access_token) {
    headers['Authorization'] = `Bearer ${authState.access_token}`;
  }

  const res = await fetch(BASE + path, { ...options, headers });

  if (res.status === 401) {
    // Try refresh
    if (authState.refresh_token) {
      try {
        const refreshRes = await fetch(BASE + '/auth/refresh', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refresh_token: authState.refresh_token })
        });
        if (refreshRes.ok) {
          const tokens = await refreshRes.json();
          auth.updateTokens(tokens.access_token, tokens.refresh_token);
          // Retry original request with new token
          headers['Authorization'] = `Bearer ${tokens.access_token}`;
          const retryRes = await fetch(BASE + path, { ...options, headers });
          if (!retryRes.ok) throw new Error(await errorMessage(retryRes));
          return retryRes.json();
        }
      } catch {
        // fall through to logout
      }
    }
    auth.logout();
    goto('/login');
    throw new Error('Unauthorized');
  }

  if (!res.ok) throw new Error(await errorMessage(res));
  if (res.status === 204) return undefined as T;
  return res.json();
}

async function errorMessage(res: Response): Promise<string> {
  try {
    const body = await res.json();
    return body.error || res.statusText;
  } catch {
    return res.statusText;
  }
}

function get_req<T>(path: string): Promise<T> {
  return request<T>(path, { method: 'GET' });
}

function post_req<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'POST', body: JSON.stringify(body) });
}

function put_req<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'PUT', body: JSON.stringify(body) });
}

function patch_req<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'PATCH', body: JSON.stringify(body) });
}

function del_req<T>(path: string): Promise<T> {
  return request<T>(path, { method: 'DELETE' });
}

// Auth
export const api = {
  auth: {
    register: (data: { email: string; username: string; display_name: string; password: string }) =>
      post_req<AuthResponse>('/auth/register', data),
    login: (data: { email: string; password: string }) =>
      post_req<AuthResponse>('/auth/login', data),
    me: () => get_req<User>('/auth/me'),
    refresh: (refresh_token: string) =>
      post_req<{ access_token: string; refresh_token: string }>('/auth/refresh', { refresh_token })
  },

  users: {
    list: () => get_req<UserSummary[]>('/users'),
    updateProfile: (data: { display_name?: string; avatar_url?: string | null }) =>
      put_req<User>('/users/me', data),
    changePassword: (data: { current_password: string; new_password: string }) =>
      post_req<{ status: string }>('/users/me/password', data)
  },

  projects: {
    list: () => get_req<Project[]>('/projects'),
    create: (data: { key: string; name: string; description?: string }) =>
      post_req<Project>('/projects', data),
    get: (key: string) => get_req<Project>(`/projects/${key}`),
    update: (key: string, data: { name?: string; description?: string }) =>
      put_req<Project>(`/projects/${key}`, data),
    getMembers: (key: string) => get_req<ProjectMember[]>(`/projects/${key}/members`),
    addMember: (key: string, data: { user_id: string; role?: string }) =>
      post_req<void>(`/projects/${key}/members`, data),
    removeMember: (key: string, userId: string) =>
      del_req<void>(`/projects/${key}/members/${userId}`)
  },

  tickets: {
    list: (key: string, params?: Record<string, string>) => {
      const qs = params ? '?' + new URLSearchParams(params).toString() : '';
      return get_req<Ticket[]>(`/projects/${key}/tickets${qs}`);
    },
    create: (key: string, data: Partial<Ticket> & { title: string; ticket_type: string }) =>
      post_req<Ticket>(`/projects/${key}/tickets`, data),
    get: (slug: string) => get_req<TicketDetail>(`/tickets/${slug}`),
    update: (slug: string, data: Partial<Ticket>) =>
      put_req<Ticket>(`/tickets/${slug}`, data),
    delete: (slug: string) => del_req<void>(`/tickets/${slug}`),
    patchStatus: (slug: string, status: string) =>
      patch_req<Ticket>(`/tickets/${slug}/status`, { status }),
    assign: (slug: string, assignee_id: string | null) =>
      patch_req<Ticket>(`/tickets/${slug}/assign`, { assignee_id }),
    getChildren: (slug: string) => get_req<TicketSummary[]>(`/tickets/${slug}/children`),
    getTags: (slug: string) => get_req<Tag[]>(`/tickets/${slug}/tags`)
  },

  comments: {
    list: (slug: string) => get_req<Comment[]>(`/tickets/${slug}/comments`),
    create: (slug: string, data: { body: string; parent_id?: string }) =>
      post_req<Comment>(`/tickets/${slug}/comments`, data),
    update: (id: string, body: string) =>
      put_req<Comment>(`/comments/${id}`, { body }),
    delete: (id: string) => del_req<void>(`/comments/${id}`)
  },

  tags: {
    list: (key: string) => get_req<Tag[]>(`/projects/${key}/tags`),
    create: (key: string, data: { name: string; color?: string }) =>
      post_req<Tag>(`/projects/${key}/tags`, data),
    delete: (key: string, id: string) => del_req<void>(`/projects/${key}/tags/${id}`),
    addToTicket: (slug: string, tagId: string) =>
      post_req<void>(`/tickets/${slug}/tags/${tagId}`, {}),
    removeFromTicket: (slug: string, tagId: string) =>
      del_req<void>(`/tickets/${slug}/tags/${tagId}`)
  },

  links: {
    create: (slug: string, data: { target_slug: string; link_type: string }) =>
      post_req<TicketLink>(`/tickets/${slug}/links`, data),
    delete: (slug: string, id: string) =>
      del_req<void>(`/tickets/${slug}/links/${id}`),
    createRepo: (slug: string, data: { repo_url: string; branch_name?: string; pr_url?: string }) =>
      post_req<RepoLink>(`/tickets/${slug}/repos`, data),
    deleteRepo: (slug: string, id: string) =>
      del_req<void>(`/tickets/${slug}/repos/${id}`)
  },

  sprints: {
    list: (key: string) => get_req<Sprint[]>(`/projects/${key}/sprints`),
    create: (key: string, data: { name: string; goal?: string; start_date?: string; end_date?: string }) =>
      post_req<Sprint>(`/projects/${key}/sprints`, data),
    update: (id: string, data: Partial<Sprint>) =>
      put_req<Sprint>(`/sprints/${id}`, data),
    start: (id: string) => patch_req<Sprint>(`/sprints/${id}/start`, {}),
    complete: (id: string) => patch_req<Sprint>(`/sprints/${id}/complete`, {})
  },

  board: {
    kanban: (key: string) => get_req<Board>(`/projects/${key}/board`),
    scrum: (key: string) => get_req<Board>(`/projects/${key}/board/scrum`),
    move: (ticket_slug: string, to_status: string) =>
      patch_req<Ticket>('/board/move', { ticket_slug, to_status })
  },

  activity: {
    list: (slug: string) => get_req<ActivityEntry[]>(`/tickets/${slug}/activity`)
  }
};
