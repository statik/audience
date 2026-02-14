import { useState } from "react";

interface PresetEditorProps {
  initialName?: string;
  onSave: (name: string) => void;
  onCancel: () => void;
}

export function PresetEditor({ initialName = "", onSave, onCancel }: PresetEditorProps) {
  const [name, setName] = useState(initialName);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      onSave(name.trim());
    }
  };

  return (
    <div className="mt-2 p-3 bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)]">
      <form onSubmit={handleSubmit}>
        <label className="block text-xs text-[var(--color-text-muted)] mb-1">
          Preset Name
        </label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g., Front Row Center"
          className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-primary)]"
          autoFocus
        />
        <div className="flex gap-2 mt-2">
          <button
            type="submit"
            disabled={!name.trim()}
            className="flex-1 px-3 py-1.5 text-sm bg-[var(--color-primary)] text-white rounded hover:bg-[var(--color-primary-hover)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Save Position
          </button>
          <button
            type="button"
            onClick={onCancel}
            className="px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
