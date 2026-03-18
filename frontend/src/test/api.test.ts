import { describe, expect, it, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { auth } from '../lib/auth';
import { api } from '../lib/api';
import * as nav from '$app/navigation';

// All modules are imported once at the top. We avoid vi.resetModules() here
// because api.ts binds $app/navigation.goto at import time — resetting modules
// would create a fresh navigation instance that the spy can't observe.

const mockUser = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'alice@example.com',
  username: 'alice',
  display_name: 'Alice',
  avatar_url: null,
  created_at: new Date().toISOString()
};

function makeFetchResponse(
  status: number,
  body: unknown,
  statusText = 'OK'
): Response {
  return {
    ok: status >= 200 && status < 300,
    status,
    statusText,
    json: async () => body,
    headers: new Headers({ 'Content-Type': 'application/json' })
  } as unknown as Response;
}

describe('api', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    auth.logout();

    fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);
  });

  it('attaches Bearer header when token present', async () => {
    auth.login(mockUser, 'my-token', 'my-refresh');
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, { id: '1' }));

    await api.auth.me();

    const [, options] = fetchMock.mock.calls[0];
    expect(options.headers['Authorization']).toBe('Bearer my-token');
  });

  it('omits Authorization when logged out', async () => {
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, []));

    await api.projects.list();

    const [, options] = fetchMock.mock.calls[0];
    expect(options.headers['Authorization']).toBeUndefined();
  });

  it('returns parsed JSON on 200', async () => {
    auth.login(mockUser, 'tok', 'ref');
    const payload = { id: 'abc', name: 'Test' };
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, payload));

    const result = await api.projects.get('TEST');
    expect(result).toEqual(payload);
  });

  it('returns undefined on 204', async () => {
    auth.login(mockUser, 'tok', 'ref');
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 204,
      statusText: 'No Content',
      json: async () => { throw new Error('No body'); },
      headers: new Headers()
    } as unknown as Response);

    const result = await api.tickets.delete('TEST-1');
    expect(result).toBeUndefined();
  });

  it('throws on 500 with error body', async () => {
    auth.login(mockUser, 'tok', 'ref');
    fetchMock.mockResolvedValueOnce(
      makeFetchResponse(500, { error: 'Internal server error' }, 'Internal Server Error')
    );

    await expect(api.projects.get('TEST')).rejects.toThrow('Internal server error');
  });

  it('throws with statusText on non-JSON error', async () => {
    auth.login(mockUser, 'tok', 'ref');
    fetchMock.mockResolvedValueOnce({
      ok: false,
      status: 503,
      statusText: 'Service Unavailable',
      json: async () => { throw new SyntaxError('not json'); },
      headers: new Headers()
    } as unknown as Response);

    await expect(api.projects.get('TEST')).rejects.toThrow('Service Unavailable');
  });

  it('401 triggers refresh attempt', async () => {
    auth.login(mockUser, 'expired-token', 'my-refresh');

    fetchMock.mockResolvedValueOnce(makeFetchResponse(401, { error: 'Unauthorized' }, 'Unauthorized'));
    fetchMock.mockResolvedValueOnce(
      makeFetchResponse(200, { access_token: 'new-token', refresh_token: 'new-refresh' })
    );
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, { id: 'u1' }));

    await api.auth.me();

    expect(fetchMock).toHaveBeenCalledTimes(3);
    const refreshCall = fetchMock.mock.calls[1];
    expect(refreshCall[0]).toContain('/auth/refresh');
  });

  it('successful refresh retries original with new token', async () => {
    auth.login(mockUser, 'old-token', 'refresh-tok');

    fetchMock.mockResolvedValueOnce(makeFetchResponse(401, {}, 'Unauthorized'));
    fetchMock.mockResolvedValueOnce(
      makeFetchResponse(200, { access_token: 'fresh-token', refresh_token: 'fresh-refresh' })
    );
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, { id: 'u1' }));

    await api.auth.me();

    const retryCall = fetchMock.mock.calls[2];
    const [, retryOpts] = retryCall;
    expect(retryOpts.headers['Authorization']).toBe('Bearer fresh-token');
  });

  it('failed refresh calls logout and goto("/login")', async () => {
    auth.login(mockUser, 'expired', 'bad-refresh');
    const gotoSpy = vi.spyOn(nav, 'goto');

    fetchMock.mockResolvedValueOnce(makeFetchResponse(401, {}, 'Unauthorized'));
    fetchMock.mockResolvedValueOnce(
      makeFetchResponse(401, { error: 'Invalid refresh token' }, 'Unauthorized')
    );

    await expect(api.auth.me()).rejects.toThrow();

    const state = get(auth);
    expect(state.access_token).toBeNull();
    expect(gotoSpy).toHaveBeenCalledWith('/login');
  });

  it('tickets.list appends query string', async () => {
    auth.login(mockUser, 'tok', 'ref');
    fetchMock.mockResolvedValueOnce(makeFetchResponse(200, []));

    await api.tickets.list('TEST', { status: 'done' });

    const [url] = fetchMock.mock.calls[0];
    expect(url).toContain('?status=done');
  });
});
