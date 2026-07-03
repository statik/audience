import { useEffect, useState } from "react";
import { useProfiles } from "../hooks/useProfiles";
import { ConfirmButton } from "./ConfirmButton";
import { PresetEditor } from "./PresetEditor";

/**
 * Switch between named preset collections (one per venue or show).
 * Creating a profile binds it to the currently active endpoint;
 * switching to it re-activates that endpoint.
 */
export function ProfileSwitcher() {
  const {
    profiles,
    activeProfileId,
    loadProfiles,
    switchProfile,
    createProfile,
    deleteProfile,
  } = useProfiles();
  const [showEditor, setShowEditor] = useState(false);

  useEffect(() => {
    loadProfiles();
  }, [loadProfiles]);

  const handleCreate = async (name: string) => {
    try {
      await createProfile(name);
      setShowEditor(false);
    } catch {
      // Failure already surfaced as a toast; keep the editor open
    }
  };

  return (
    <div className="p-3 border-b border-[var(--color-border)]">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold text-[var(--color-text)] uppercase tracking-wide">
          Profile
        </h3>
        <div className="flex items-center gap-2">
          <button
            className="text-xs text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] transition-colors"
            onClick={() => setShowEditor(true)}
          >
            + New
          </button>
          {activeProfileId && profiles.length > 1 && (
            <ConfirmButton
              className="text-xs text-[var(--color-danger)] hover:text-red-400 transition-colors"
              label="Delete"
              onConfirm={() => deleteProfile(activeProfileId)}
              aria-label="Delete active profile"
            />
          )}
        </div>
      </div>

      <select
        value={activeProfileId ?? ""}
        onChange={(e) => switchProfile(e.target.value)}
        aria-label="Active profile"
        className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
      >
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.name}
          </option>
        ))}
      </select>

      {showEditor && (
        <PresetEditor
          label="Profile Name"
          placeholder="e.g., Main Hall"
          saveLabel="Create Profile"
          onSave={handleCreate}
          onCancel={() => setShowEditor(false)}
        />
      )}
    </div>
  );
}
