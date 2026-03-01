import { useState } from 'react';

export default function PasswordChangeForm() {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setError('');
    setMessage('');

    if (newPassword !== confirmPassword) {
      setError('New passwords do not match');
      return;
    }

    setSaving(true);

    try {
      const res = await fetch('/api/proxy/profile/password', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword,
        }),
      });

      if (res.ok) {
        setMessage('Password updated successfully');
        setCurrentPassword('');
        setNewPassword('');
        setConfirmPassword('');
      } else {
        const body = await res.json();
        setError(body?.error?.message ?? 'Failed to change password');
      }
    } catch {
      setError('Failed to change password');
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      <div>
        <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Current Password</label>
        <input
          type="password"
          value={currentPassword}
          onChange={(e) => setCurrentPassword(e.target.value)}
          className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
          required
        />
      </div>
      <div>
        <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">New Password</label>
        <input
          type="password"
          value={newPassword}
          onChange={(e) => setNewPassword(e.target.value)}
          className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
          minLength={8}
          required
        />
      </div>
      <div>
        <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Confirm New Password</label>
        <input
          type="password"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
          minLength={8}
          required
        />
      </div>
      {error && <p className="text-xs text-red-400">{error}</p>}
      {message && <p className="text-xs text-[rgb(var(--primary))]">{message}</p>}
      <div className="flex justify-end">
        <button
          type="submit"
          disabled={saving}
          className="px-3 py-1.5 bg-[rgb(var(--primary))] text-[rgb(var(--primary-foreground))] text-xs font-medium rounded-md hover:opacity-90 disabled:opacity-50 transition-opacity"
        >
          {saving ? 'Updating...' : 'Update Password'}
        </button>
      </div>
    </form>
  );
}
