import { useState } from 'react';

interface Props {
  flagId: string;
  envId: string;
  projectId: string;
  initialPercentage: number;
  sticky: boolean;
}

export default function PercentageSlider({ flagId, envId, projectId, initialPercentage, sticky }: Props) {
  const [percentage, setPercentage] = useState(initialPercentage);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  async function save() {
    setSaving(true);
    try {
      const res = await fetch(
        `/api/proxy/projects/${projectId}/flags/${flagId}/environments/${envId}`,
        {
          method: 'PATCH',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ gate_config: { percentage, sticky } }),
        },
      );
      if (res.ok) {
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      }
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-4">
        <div className="flex-1 relative">
          <div className="w-full bg-[rgb(var(--muted))] rounded-full h-1.5">
            <div
              className="bg-[rgb(var(--primary))] h-1.5 rounded-full transition-all"
              style={{ width: `${percentage}%` }}
            ></div>
          </div>
          <input
            type="range"
            min={0}
            max={100}
            value={percentage}
            onChange={(e) => setPercentage(Number(e.target.value))}
            className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
          />
        </div>
        <span className="font-mono text-xs font-medium text-[rgb(var(--foreground))] w-10 text-right">{percentage}%</span>
      </div>
      <div className="flex items-center gap-3">
        <button
          onClick={save}
          disabled={saving}
          className="px-3 py-1.5 bg-[rgb(var(--primary))] text-[rgb(var(--primary-foreground))] text-xs font-medium rounded-md hover:opacity-90 disabled:opacity-50 transition-opacity"
        >
          {saving ? 'Saving...' : 'Save'}
        </button>
        {saved && <span className="text-xs text-[rgb(var(--status-active))]">Saved</span>}
      </div>
    </div>
  );
}
