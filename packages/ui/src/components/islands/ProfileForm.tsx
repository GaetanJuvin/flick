import { useState } from 'react';

interface Props {
  currentName: string;
  currentEmail: string;
}

export default function ProfileForm({ currentName, currentEmail }: Props) {
  const [name, setName] = useState(currentName);
  const [email, setEmail] = useState(currentEmail);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');
    setMessage('');

    try {
      const res = await fetch('/api/proxy/profile', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, email }),
      });

      if (res.ok) {
        setMessage('Profile updated');
        // Reload after a short delay to update the header and session
        setTimeout(() => window.location.reload(), 500);
      } else {
        const body = await res.json();
        setError(body?.error?.message ?? 'Failed to update profile');
      }
    } catch {
      setError('Failed to update profile');
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      <div>
        <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
          required
        />
      </div>
      <div>
        <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Email</label>
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
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
          {saving ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </form>
  );
}
