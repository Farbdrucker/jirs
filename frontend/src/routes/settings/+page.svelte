<script lang="ts">
  import { api } from '$lib/api';
  import { auth } from '$lib/auth';

  // Profile form
  let displayName = $state($auth.user?.display_name ?? '');
  let avatarUrl = $state($auth.user?.avatar_url ?? '');
  let savingProfile = $state(false);
  let profileMsg = $state('');
  let profileError = $state('');

  // Password form
  let currentPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let savingPassword = $state(false);
  let passwordMsg = $state('');
  let passwordError = $state('');

  async function saveProfile() {
    profileMsg = '';
    profileError = '';
    savingProfile = true;
    try {
      const updated = await api.users.updateProfile({
        display_name: displayName || undefined,
        avatar_url: avatarUrl || null
      });
      auth.updateUser(updated);
      profileMsg = 'Saved';
    } catch (e: any) {
      profileError = e.message;
    } finally {
      savingProfile = false;
    }
  }

  async function changePassword() {
    passwordMsg = '';
    passwordError = '';
    if (newPassword !== confirmPassword) {
      passwordError = 'Passwords do not match';
      return;
    }
    if (newPassword.length < 6) {
      passwordError = 'Password must be at least 6 characters';
      return;
    }
    savingPassword = true;
    try {
      await api.users.changePassword({ current_password: currentPassword, new_password: newPassword });
      passwordMsg = 'Password changed';
      currentPassword = newPassword = confirmPassword = '';
    } catch (e: any) {
      passwordError = e.message;
    } finally {
      savingPassword = false;
    }
  }
</script>

<div class="max-w-xl mx-auto">
  <h1 class="text-2xl font-bold text-gray-900 mb-6">Settings</h1>

  <!-- Profile Card -->
  <div class="card p-6 mb-6">
    <h2 class="text-lg font-semibold mb-4">Profile</h2>
    <form onsubmit={(e) => { e.preventDefault(); saveProfile(); }}>
      <div class="mb-4">
        <label class="label" for="display_name">Display name</label>
        <input id="display_name" class="input" bind:value={displayName} placeholder="Your name" required />
      </div>
      <div class="mb-4">
        <label class="label" for="avatar_url">Avatar URL (optional)</label>
        <input id="avatar_url" class="input" bind:value={avatarUrl} placeholder="https://…" />
      </div>
      {#if profileError}
        <p class="text-red-600 text-sm mb-3">{profileError}</p>
      {/if}
      {#if profileMsg}
        <p class="text-green-600 text-sm mb-3">{profileMsg}</p>
      {/if}
      <button type="submit" class="btn-primary" disabled={savingProfile}>
        {savingProfile ? 'Saving…' : 'Save profile'}
      </button>
    </form>
  </div>

  <!-- Password Card -->
  <div class="card p-6">
    <h2 class="text-lg font-semibold mb-4">Change Password</h2>
    <form onsubmit={(e) => { e.preventDefault(); changePassword(); }}>
      <div class="mb-4">
        <label class="label" for="current_password">Current password</label>
        <input id="current_password" type="password" class="input" bind:value={currentPassword} required />
      </div>
      <div class="mb-4">
        <label class="label" for="new_password">New password</label>
        <input id="new_password" type="password" class="input" bind:value={newPassword} required />
      </div>
      <div class="mb-4">
        <label class="label" for="confirm_password">Confirm new password</label>
        <input id="confirm_password" type="password" class="input" bind:value={confirmPassword} required />
      </div>
      {#if passwordError}
        <p class="text-red-600 text-sm mb-3">{passwordError}</p>
      {/if}
      {#if passwordMsg}
        <p class="text-green-600 text-sm mb-3">{passwordMsg}</p>
      {/if}
      <button type="submit" class="btn-primary" disabled={savingPassword}>
        {savingPassword ? 'Changing…' : 'Change password'}
      </button>
    </form>
  </div>
</div>
