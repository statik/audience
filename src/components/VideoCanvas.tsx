import { useCallback, useEffect, useRef } from "react";
import { useVideoFeed } from "../hooks/useVideoFeed";
import { usePtzControl } from "../hooks/usePtzControl";
import { useAppStore } from "../store/app-store";
import { PresetOverlay } from "./PresetOverlay";

export function VideoCanvas() {
  const { videoRef, error } = useVideoFeed();
  const { handleVideoClick, handleVideoScroll } = usePtzControl();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const mode = useAppStore((s) => s.mode);
  const isConnected = useAppStore((s) => s.isConnected);

  // Sync canvas size to container
  useEffect(() => {
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        if (canvasRef.current) {
          canvasRef.current.width = width;
          canvasRef.current.height = height;
        }
      }
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => resizeObserver.disconnect();
  }, []);

  const onCanvasClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!canvasRef.current) return;
      const rect = canvasRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (mode === "operation") {
        handleVideoClick(x, y, rect.width, rect.height);
      }
    },
    [mode, handleVideoClick]
  );

  const onCanvasWheel = useCallback(
    (e: React.WheelEvent<HTMLCanvasElement>) => {
      e.preventDefault();
      handleVideoScroll(e.deltaY);
    },
    [handleVideoScroll]
  );

  return (
    <div
      ref={containerRef}
      className="relative w-full h-full bg-black overflow-hidden"
    >
      {/* Video element */}
      <video
        ref={videoRef}
        autoPlay
        playsInline
        muted
        className="absolute inset-0 w-full h-full object-contain"
      />

      {/* No signal indicator */}
      {!isConnected && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center">
            <div className="text-2xl font-bold text-[var(--color-text-muted)] mb-2">
              No Signal
            </div>
            <div className="text-sm text-[var(--color-text-muted)]">
              {error || "Select a video source to begin"}
            </div>
          </div>
        </div>
      )}

      {/* Overlay canvas for presets */}
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full cursor-crosshair"
        onClick={onCanvasClick}
        onWheel={onCanvasWheel}
      />

      {/* Preset overlays rendered as HTML elements */}
      <PresetOverlay canvasRef={canvasRef} />
    </div>
  );
}
