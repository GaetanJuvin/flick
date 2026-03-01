import { useState } from 'react';

interface Props {
  projectId: string;
}

export default function FlagCreateModal({ projectId }: Props) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState('');
  const [key, setKey] = useState('');
  const [gateType, setGateType] = useState('boolean');
  const [description, setDescription] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  function generateKey(name: string) {
    return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');

    try {
      const res = await fetch(`/api/proxy/projects/${projectId}/flags`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, name, gate_type: gateType, description }),
      });

      if (res.ok) {
        window.location.reload();
      } else {
        const body = await res.json();
        setError(body?.error?.message ?? 'Failed to create flag');
      }
    } catch {
      setError('Failed to create flag');
    } finally {
      setSaving(false);
    }
  }

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="px-3 py-1.5 bg-[rgb(var(--primary))] text-[rgb(var(--primary-foreground))] text-xs font-medium rounded-md hover:opacity-90 transition-opacity"
      >
        + New Flag
      </button>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={() => setOpen(false)}>
      <div className="rounded-lg border border-[rgb(var(--border))] bg-[rgb(var(--card))] shadow-xl w-full max-w-md p-6" onClick={e => e.stopPropagation()}>
        <h2 className="text-sm font-semibold text-[rgb(var(--foreground))] mb-4">Create Flag</h2>
        <form onSubmit={submit} className="space-y-4">
          <div>
            <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => { setName(e.target.value); setKey(generateKey(e.target.value)); }}
              className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
              placeholder="e.g. Dark Mode"
              required
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Key</label>
            <input
              type="text"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm font-mono focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
              pattern="[a-z0-9-]+"
              required
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Gate Type</label>
            <select
              value={gateType}
              onChange={(e) => setGateType(e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
            >
              <option value="boolean">Boolean</option>
              <option value="percentage">Percentage</option>
              <option value="group">Group</option>
            </select>
          </div>
          <div>
            <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Description</label>
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
              placeholder="Optional description"
            />
          </div>
          {error && <p className="text-xs text-red-400">{error}</p>}
          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="px-3 py-1.5 text-xs text-[rgb(var(--muted-foreground))] hover:text-[rgb(var(--foreground))] rounded-md transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="px-3 py-1.5 bg-[rgb(var(--primary))] text-[rgb(var(--primary-foreground))] text-xs font-medium rounded-md hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              {saving ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
