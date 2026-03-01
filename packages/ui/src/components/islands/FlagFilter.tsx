import { useState } from 'react';

interface Props {
  initialSearch?: string;
  initialStatus?: string;
}

const tabs = [
  { value: '', label: 'All' },
  { value: 'active', label: 'Active' },
  { value: 'rolling-out', label: 'Rolling Out' },
  { value: 'inactive', label: 'Inactive' },
];

export default function FlagFilter({ initialSearch = '', initialStatus = '' }: Props) {
  const [search, setSearch] = useState(initialSearch);

  function apply() {
    const params = new URLSearchParams(window.location.search);
    if (search) params.set('search', search);
    else params.delete('search');
    window.location.search = params.toString();
  }

  function setStatus(status: string) {
    const params = new URLSearchParams(window.location.search);
    if (status) params.set('status', status);
    else params.delete('status');
    window.location.search = params.toString();
  }

  return (
    <div className="flex items-center justify-between gap-4 mb-4">
      <div className="flex items-center gap-1">
        <div className="relative">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[rgb(var(--muted-foreground))]"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
          </svg>
          <input
            type="text"
            placeholder="Search flags by key, name, or tag..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && apply()}
            className="pl-9 pr-3 py-1.5 w-72 rounded-md border text-xs bg-transparent border-[rgb(var(--border))] text-[rgb(var(--foreground))] placeholder:text-[rgb(var(--muted-foreground))] focus:outline-none focus:ring-1 focus:ring-[rgb(var(--primary))]"
          />
        </div>
      </div>

      <div className="flex items-center gap-1 rounded-lg border border-[rgb(var(--border))] p-0.5">
        {tabs.map((tab) => (
          <button
            key={tab.value}
            onClick={() => setStatus(tab.value)}
            className={`px-3 py-1 rounded-md text-xs font-medium transition-colors ${
              initialStatus === tab.value
                ? 'bg-[rgb(var(--accent))] text-[rgb(var(--foreground))]'
                : 'text-[rgb(var(--muted-foreground))] hover:text-[rgb(var(--foreground))]'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
    </div>
  );
}
