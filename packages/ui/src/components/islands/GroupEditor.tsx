import { useState } from 'react';

interface Rule {
  attribute: string;
  operator: string;
  value: string;
}

interface Props {
  projectId: string;
  groupId?: string;
  initialName?: string;
  initialSlug?: string;
  initialDescription?: string;
  initialRules?: Rule[];
}

const OPERATORS = [
  { value: 'eq', label: 'equals' },
  { value: 'neq', label: 'not equals' },
  { value: 'in', label: 'in' },
  { value: 'not_in', label: 'not in' },
  { value: 'contains', label: 'contains' },
  { value: 'starts_with', label: 'starts with' },
  { value: 'ends_with', label: 'ends with' },
  { value: 'gt', label: '>' },
  { value: 'gte', label: '>=' },
  { value: 'lt', label: '<' },
  { value: 'lte', label: '<=' },
  { value: 'regex', label: 'regex' },
];

export default function GroupEditor({ projectId, groupId, initialName = '', initialSlug = '', initialDescription = '', initialRules = [] }: Props) {
  const [name, setName] = useState(initialName);
  const [slug, setSlug] = useState(initialSlug);
  const [description, setDescription] = useState(initialDescription);
  const [rules, setRules] = useState<Rule[]>(
    initialRules.length > 0 ? initialRules : [{ attribute: '', operator: 'eq', value: '' }],
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  function addRule() {
    setRules([...rules, { attribute: '', operator: 'eq', value: '' }]);
  }

  function removeRule(index: number) {
    setRules(rules.filter((_, i) => i !== index));
  }

  function updateRule(index: number, field: keyof Rule, value: string) {
    const updated = [...rules];
    updated[index] = { ...updated[index], [field]: value };
    setRules(updated);
  }

  function generateSlug(name: string) {
    return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  }

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    setError('');

    const parsedRules = rules.map((r) => ({
      attribute: r.attribute,
      operator: r.operator,
      value: ['in', 'not_in'].includes(r.operator) ? r.value.split(',').map((v) => v.trim()) : r.value,
    }));

    const body = groupId
      ? { name, description, rules: parsedRules }
      : { name, slug, description, rules: parsedRules };

    try {
      const url = groupId
        ? `/api/proxy/projects/${projectId}/groups/${groupId}`
        : `/api/proxy/projects/${projectId}/groups`;

      const res = await fetch(url, {
        method: groupId ? 'PATCH' : 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });

      if (res.ok) {
        window.location.href = '/groups';
      } else {
        const data = await res.json();
        setError(data?.error?.message ?? 'Failed to save group');
      }
    } catch {
      setError('Failed to save group');
    } finally {
      setSaving(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-6 max-w-2xl">
      <div className="rounded-lg border border-[rgb(var(--border))] bg-[rgb(var(--card))] p-5 space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => { setName(e.target.value); if (!groupId) setSlug(generateSlug(e.target.value)); }}
              className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
              required
            />
          </div>
          {!groupId && (
            <div>
              <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Slug</label>
              <input
                type="text"
                value={slug}
                onChange={(e) => setSlug(e.target.value)}
                className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm font-mono focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
                required
              />
            </div>
          )}
        </div>
        <div>
          <label className="block text-xs font-medium text-[rgb(var(--muted-foreground))] mb-1">Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            className="w-full px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
            rows={2}
          />
        </div>
      </div>

      <div className="rounded-lg border border-[rgb(var(--border))] bg-[rgb(var(--card))] p-5">
        <div className="flex items-center justify-between mb-3">
          <label className="text-xs font-medium uppercase tracking-wider text-[rgb(var(--muted-foreground))]">Rules (ANDed together)</label>
          <button type="button" onClick={addRule} className="text-xs text-[rgb(var(--primary))] hover:opacity-80">
            + Add Rule
          </button>
        </div>
        <div className="space-y-2">
          {rules.map((rule, i) => (
            <div key={i} className="flex gap-2 items-center">
              <input
                type="text"
                placeholder="attribute"
                value={rule.attribute}
                onChange={(e) => updateRule(i, 'attribute', e.target.value)}
                className="flex-1 px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm font-mono focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))] placeholder:text-[rgb(var(--muted-foreground))/50]"
                required
              />
              <select
                value={rule.operator}
                onChange={(e) => updateRule(i, 'operator', e.target.value)}
                className="px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
              >
                {OPERATORS.map((op) => (
                  <option key={op.value} value={op.value}>{op.label}</option>
                ))}
              </select>
              <input
                type="text"
                placeholder="value"
                value={rule.value}
                onChange={(e) => updateRule(i, 'value', e.target.value)}
                className="flex-1 px-3 py-2 rounded-md border border-[rgb(var(--border))] bg-[rgb(var(--background))] text-[rgb(var(--foreground))] text-sm font-mono focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))] placeholder:text-[rgb(var(--muted-foreground))/50]"
                required
              />
              {rules.length > 1 && (
                <button
                  type="button"
                  onClick={() => removeRule(i)}
                  className="text-xs text-[rgb(var(--status-inactive))] hover:text-[rgb(var(--foreground))] transition-colors"
                >
                  Remove
                </button>
              )}
            </div>
          ))}
        </div>
      </div>

      {error && <p className="text-xs text-red-400">{error}</p>}

      <button
        type="submit"
        disabled={saving}
        className="px-3 py-1.5 bg-[rgb(var(--primary))] text-[rgb(var(--primary-foreground))] text-xs font-medium rounded-md hover:opacity-90 disabled:opacity-50 transition-opacity"
      >
        {saving ? 'Saving...' : (groupId ? 'Update Group' : 'Create Group')}
      </button>
    </form>
  );
}
