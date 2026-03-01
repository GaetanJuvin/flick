import { useState } from 'react';

interface Props {
  flagId: string;
  envId: string;
  projectId: string;
  enabled: boolean;
}

export default function FlagToggle({ flagId, envId, projectId, enabled: initial }: Props) {
  const [enabled, setEnabled] = useState(initial);
  const [loading, setLoading] = useState(false);

  async function toggle() {
    setLoading(true);
    setEnabled(!enabled); // Optimistic update

    try {
      const res = await fetch(
        `/api/proxy/projects/${projectId}/flags/${flagId}/environments/${envId}/toggle`,
        { method: 'POST' },
      );
      if (!res.ok) {
        setEnabled(enabled); // Revert on failure
      }
    } catch {
      setEnabled(enabled); // Revert on failure
    } finally {
      setLoading(false);
    }
  }

  return (
    <button
      type="button"
      onClick={toggle}
      disabled={loading}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
        enabled ? 'bg-[rgb(var(--status-active))]' : 'bg-[rgb(var(--muted))]'
      } ${loading ? 'opacity-50 cursor-wait' : 'cursor-pointer'}`}
      role="switch"
      aria-checked={enabled}
    >
      <span
        className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${
          enabled ? 'translate-x-[18px]' : 'translate-x-[3px]'
        }`}
      />
    </button>
  );
}
