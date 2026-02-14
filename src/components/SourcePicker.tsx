import { useCallback, useEffect, useState } from "react";
import { useAppStore } from "../store/app-store";
import { useVideoFeed } from "../hooks/useVideoFeed";
import type { VideoSource } from "@shared/types";

export function SourcePicker() {
  const videoSources = useAppStore((s) => s.videoSources);
  const setVideoSources = useAppStore((s) => s.setVideoSources);
  const activeVideoSource = useAppStore((s) => s.activeVideoSource);
  const setActiveVideoSource = useAppStore((s) => s.setActiveVideoSource);
  const { enumerateDevices } = useVideoFeed();
  const [isOpen, setIsOpen] = useState(false);

  const refreshSources = useCallback(async () => {
    const localDevices = await enumerateDevices();
    // NDI sources would come from backend invoke
    setVideoSources(localDevices);
  }, [enumerateDevices, setVideoSources]);

  useEffect(() => {
    refreshSources();
  }, [refreshSources]);

  const selectSource = (source: VideoSource) => {
    setActiveVideoSource(source);
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <button
        className="flex items-center gap-2 px-3 py-1.5 text-sm rounded border border-[var(--color-border)] bg-[var(--color-bg-dark)] text-[var(--color-text)] hover:border-[var(--color-primary)] transition-colors"
        onClick={() => {
          refreshSources();
          setIsOpen(!isOpen);
        }}
      >
        <span className="truncate max-w-[180px]">
          {activeVideoSource?.label || "Select Source"}
        </span>
        <span className="text-xs">&#9660;</span>
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-1 w-64 bg-[var(--color-bg-panel)] border border-[var(--color-border)] rounded-lg shadow-xl z-50">
          {videoSources.length === 0 ? (
            <div className="px-3 py-2 text-sm text-[var(--color-text-muted)]">
              No video sources found
            </div>
          ) : (
            videoSources.map((source) => (
              <button
                key={source.id}
                className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--color-bg-card)] transition-colors ${
                  activeVideoSource?.id === source.id
                    ? "text-[var(--color-primary)]"
                    : "text-[var(--color-text)]"
                }`}
                onClick={() => selectSource(source)}
              >
                <div className="truncate">{source.label}</div>
                <div className="text-xs text-[var(--color-text-muted)]">
                  {source.type === "local" ? "Local Device" : "NDI Source"}
                </div>
              </button>
            ))
          )}
          <button
            className="w-full text-left px-3 py-2 text-sm text-[var(--color-text-muted)] hover:bg-[var(--color-bg-card)] border-t border-[var(--color-border)] transition-colors"
            onClick={refreshSources}
          >
            Refresh Sources
          </button>
        </div>
      )}
    </div>
  );
}
