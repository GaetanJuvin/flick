import { useState, type ReactNode } from 'react';

interface Props {
  title: string;
  message: string;
  confirmLabel?: string;
  onConfirm: () => Promise<void>;
  children: ReactNode;
  variant?: 'danger' | 'default';
}

export default function ConfirmDialog({ title, message, confirmLabel = 'Confirm', onConfirm, children, variant = 'danger' }: Props) {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  async function handleConfirm() {
    setLoading(true);
    try {
      await onConfirm();
      setOpen(false);
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <span onClick={() => setOpen(true)}>{children}</span>
      {open && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-sm p-6">
            <h3 className="text-lg font-semibold">{title}</h3>
            <p className="mt-2 text-sm text-gray-600">{message}</p>
            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setOpen(false)}
                className="px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                disabled={loading}
                className={`px-4 py-2 text-sm text-white rounded-md disabled:opacity-50 ${
                  variant === 'danger' ? 'bg-red-600 hover:bg-red-700' : 'bg-blue-600 hover:bg-blue-700'
                }`}
              >
                {loading ? 'Working...' : confirmLabel}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
