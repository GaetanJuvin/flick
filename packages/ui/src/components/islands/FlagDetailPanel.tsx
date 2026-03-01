import { useState, useEffect } from 'react';

interface FlagEnv {
  id: string;
  environment_id: string;
  environment_slug: string;
  enabled: boolean;
  gate_config: Record<string, unknown>;
}

interface Flag {
  id: string;
  key: string;
  name: string;
  description: string;
  gate_type: string;
  tags: string[];
  archived: boolean;
  project_id: string;
  created_at: string;
  updated_at: string;
  environments?: FlagEnv[];
}

interface Props {
  flags: Flag[];
  environments: { id: string; slug: string; name: string }[];
  projectId: string;
}

export default function FlagDetailPanel({ flags, environments, projectId }: Props) {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const selected = flags.find(f => f.key === selectedKey);

  // Listen for row clicks dispatched via custom events
  useEffect(() => {
    function handleSelect(e: CustomEvent) {
      setSelectedKey(e.detail.key);
    }
    window.addEventListener('flag-select', handleSelect as EventListener);
    return () => window.removeEventListener('flag-select', handleSelect as EventListener);
  }, []);

  if (!selected) {
    return (
      <div className="rounded-lg border border-[rgb(var(--border))] bg-[rgb(var(--card))] p-6 text-center">
        <p className="text-xs font-medium uppercase tracking-wider text-[rgb(var(--muted-foreground))]">Select a flag</p>
        <p className="mt-1 text-xs text-[rgb(var(--muted-foreground))/60]">Click a row in the table to view detailed flag information</p>
      </div>
    );
  }

  const statusLabel = selected.archived ? 'Archived' : (selected.environments?.some(e => e.enabled) ? 'Active' : 'Inactive');
  const statusClass = selected.archived
    ? 'bg-[rgb(var(--muted))] text-[rgb(var(--status-inactive))]'
    : selected.environments?.some(e => e.enabled)
      ? 'bg-[rgb(var(--status-active)/.1)] text-[rgb(var(--status-active))]'
      : 'bg-[rgb(var(--muted))] text-[rgb(var(--status-inactive))]';

  const rolloutPct = selected.gate_type === 'percentage'
    ? (selected.environments?.find(e => e.enabled)?.gate_config?.percentage as number ?? 0) + '%'
    : selected.environments?.some(e => e.enabled) ? '100%' : '0%';

  return (
    <div className="rounded-lg border border-[rgb(var(--border))] bg-[rgb(var(--card))] p-5 space-y-5">
      {/* Header */}
      <div>
        <span className="font-mono text-xs text-[rgb(var(--foreground))]">{selected.key}</span>
        <h3 className="text-sm font-semibold text-[rgb(var(--foreground))] mt-0.5">{selected.name}</h3>
      </div>

      <div className="flex items-center gap-2">
        <span className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-xs font-medium ${statusClass}`}>
          <span className={`h-1.5 w-1.5 rounded-full ${selected.environments?.some(e => e.enabled) ? 'bg-[rgb(var(--status-active))]' : 'bg-[rgb(var(--status-inactive))]'}`}></span>
          {statusLabel}
        </span>
      </div>

      {/* Description */}
      {selected.description && (
        <p className="text-xs text-[rgb(var(--muted-foreground))]">{selected.description}</p>
      )}

      {/* Rollout */}
      <div className="space-y-2">
        <p className="text-xs font-medium uppercase tracking-wider text-[rgb(var(--muted-foreground))]">Rollout</p>
        <div className="w-full bg-[rgb(var(--muted))] rounded-full h-1.5">
          <div
            className="bg-[rgb(var(--primary))] h-1.5 rounded-full transition-all"
            style={{ width: rolloutPct }}
          ></div>
        </div>
        <span className="font-mono text-xs text-[rgb(var(--foreground))]">{rolloutPct}</span>
      </div>

      {/* Environments */}
      <div className="space-y-2">
        <p className="text-xs font-medium uppercase tracking-wider text-[rgb(var(--muted-foreground))]">Environments</p>
        <div className="space-y-1.5">
          {environments.map(env => {
            const flagEnv = selected.environments?.find(fe => fe.environment_id === env.id);
            const enabled = flagEnv?.enabled ?? false;
            return (
              <div key={env.id} className="flex items-center justify-between text-xs">
                <span className="text-[rgb(var(--muted-foreground))]">{env.name}</span>
                <span className={`font-mono ${enabled ? 'text-[rgb(var(--status-active))]' : 'text-[rgb(var(--muted-foreground))/50]'}`}>
                  {enabled ? 'ON' : 'OFF'}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Metadata */}
      <div className="space-y-2 pt-2 border-t border-[rgb(var(--border))]">
        <div className="flex items-center justify-between text-xs">
          <span className="text-[rgb(var(--muted-foreground))]">Gate type</span>
          <span className="font-mono text-[rgb(var(--foreground))]">{selected.gate_type}</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-[rgb(var(--muted-foreground))]">Created</span>
          <span className="font-mono text-[rgb(var(--foreground))]">{new Date(selected.created_at).toLocaleDateString()}</span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className="text-[rgb(var(--muted-foreground))]">Updated</span>
          <span className="font-mono text-[rgb(var(--foreground))]">{new Date(selected.updated_at).toLocaleDateString()}</span>
        </div>
      </div>

      {/* Tags */}
      {selected.tags?.length > 0 && (
        <div className="space-y-2">
          <p className="text-xs font-medium uppercase tracking-wider text-[rgb(var(--muted-foreground))]">Tags</p>
          <div className="flex flex-wrap gap-1">
            {selected.tags.map(tag => (
              <span key={tag} className="inline-flex items-center rounded-full bg-[rgb(var(--secondary))] text-[rgb(var(--secondary-foreground))] text-[10px] px-1.5 py-0 h-4 font-normal">
                {tag}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Action */}
      <a
        href={`/flags/${selected.key}`}
        className="block w-full text-center px-3 py-1.5 rounded-md border border-[rgb(var(--border))] text-xs font-medium text-[rgb(var(--foreground))] hover:bg-[rgb(var(--accent))] transition-colors"
      >
        View Details
      </a>
    </div>
  );
}
