import { useState, type ReactNode } from 'react';

interface Environment {
  id: string;
  name: string;
  slug: string;
  color: string;
}

interface Props {
  environments: Environment[];
  initialEnvId?: string;
  children?: ReactNode;
}

export default function EnvironmentTabs({ environments, initialEnvId }: Props) {
  const [activeEnvId, setActiveEnvId] = useState(initialEnvId ?? environments[0]?.id);

  function selectEnv(envId: string) {
    const params = new URLSearchParams(window.location.search);
    params.set('env', envId);
    window.location.href = `${window.location.pathname}?${params.toString()}`;
  }

  return (
    <div className="border-b border-[rgb(var(--border))]">
      <nav className="flex gap-0" aria-label="Environments">
        {environments.map((env) => {
          const active = env.id === activeEnvId;
          return (
            <button
              key={env.id}
              onClick={() => selectEnv(env.id)}
              className={`px-4 py-2 text-xs font-medium border-b-2 transition-colors ${
                active
                  ? 'border-[rgb(var(--primary))] text-[rgb(var(--primary))]'
                  : 'border-transparent text-[rgb(var(--muted-foreground))] hover:text-[rgb(var(--foreground))] hover:border-[rgb(var(--border))]'
              }`}
            >
              {env.name}
            </button>
          );
        })}
      </nav>
    </div>
  );
}
