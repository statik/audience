import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import type { VideoSource } from "@shared/types";

const RECONNECT_INTERVAL_MS = 5000;

export function useVideoFeed() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const activeSource = useAppStore((s) => s.activeVideoSource);
  const setIsConnected = useAppStore((s) => s.setIsConnected);
  const setConnectionLabel = useAppStore((s) => s.setConnectionLabel);
  const setFps = useAppStore((s) => s.setFps);
  const [error, setError] = useState<string | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const clearReconnectTimer = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearInterval(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
  }, []);

  const stopCurrentStream = useCallback(() => {
    clearReconnectTimer();
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
    if (videoRef.current) {
      videoRef.current.srcObject = null;
      videoRef.current.src = "";
    }
  }, [clearReconnectTimer]);

  const connectToLocalDevice = useCallback(
    async (deviceId: string) => {
      try {
        stopCurrentStream();
        const stream = await navigator.mediaDevices.getUserMedia({
          video: { deviceId: { exact: deviceId } },
        });
        streamRef.current = stream;
        if (videoRef.current) {
          videoRef.current.srcObject = stream;
          // Wait for video to actually start playing
          videoRef.current.onloadeddata = () => {
            setIsConnected(true);
            setError(null);
          };
        }
      } catch (err) {
        setError(`Failed to access device: ${err}`);
        setIsConnected(false);
      }
    },
    [stopCurrentStream, setIsConnected]
  );

  const connectToMjpeg = useCallback(
    (url: string) => {
      stopCurrentStream();
      if (videoRef.current) {
        videoRef.current.src = url;
        // Don't set connected until data actually loads
        videoRef.current.onloadeddata = () => {
          setIsConnected(true);
          setError(null);
        };
        videoRef.current.onerror = () => {
          setIsConnected(false);
          setError("MJPEG stream connection failed");
        };
      }
    },
    [stopCurrentStream, setIsConnected]
  );

  // Connect when source changes
  useEffect(() => {
    if (!activeSource) {
      stopCurrentStream();
      setIsConnected(false);
      setConnectionLabel("No source selected");
      return;
    }

    setConnectionLabel(activeSource.label);

    const connectToSource = async () => {
      if (activeSource.type === "local" && activeSource.deviceId) {
        await connectToLocalDevice(activeSource.deviceId);
      } else if (activeSource.type === "ndi" && activeSource.ndiName) {
        // Start MJPEG stream from backend and use returned port
        try {
          const port = await invoke<number>("start_mjpeg_stream");
          const mjpegUrl = `http://127.0.0.1:${port}/stream`;
          connectToMjpeg(mjpegUrl);
        } catch (err) {
          setError(`Failed to start NDI stream: ${err}`);
          setIsConnected(false);
        }
      }
    };

    connectToSource();

    return () => {
      stopCurrentStream();
    };
  }, [activeSource, connectToLocalDevice, connectToMjpeg, stopCurrentStream, setIsConnected, setConnectionLabel]);

  // Auto-reconnect on disconnect (PRD VF-7)
  useEffect(() => {
    if (!videoRef.current) return;
    const video = videoRef.current;

    const handleDisconnect = () => {
      setIsConnected(false);
      setError("Video source disconnected. Reconnecting...");

      // Start reconnect timer
      const source = useAppStore.getState().activeVideoSource;
      if (!source) return;

      clearReconnectTimer();
      reconnectTimerRef.current = setInterval(async () => {
        // Check if the source has changed since disconnect
        const currentSource = useAppStore.getState().activeVideoSource;
        if (!currentSource || currentSource.id !== source.id) {
          clearReconnectTimer();
          return;
        }
        if (source.type === "local" && source.deviceId) {
          try {
            const stream = await navigator.mediaDevices.getUserMedia({
              video: { deviceId: { exact: source.deviceId } },
            });
            // Re-check source hasn't changed during async getUserMedia
            const latestSource = useAppStore.getState().activeVideoSource;
            if (!latestSource || latestSource.id !== source.id) {
              stream.getTracks().forEach((t) => t.stop());
              clearReconnectTimer();
              return;
            }
            clearReconnectTimer();
            streamRef.current = stream;
            if (videoRef.current) {
              videoRef.current.srcObject = stream;
              setIsConnected(true);
              setError(null);
            }
          } catch {
            // Retry on next interval
          }
        }
      }, RECONNECT_INTERVAL_MS);
    };

    video.addEventListener("ended", handleDisconnect);
    video.addEventListener("error", handleDisconnect);

    return () => {
      video.removeEventListener("ended", handleDisconnect);
      video.removeEventListener("error", handleDisconnect);
      clearReconnectTimer();
    };
  }, [setIsConnected, clearReconnectTimer]);

  // FPS counter using requestAnimationFrame
  useEffect(() => {
    if (!videoRef.current) return;
    let lastTime = performance.now();
    let frameCount = 0;
    let lastVideoTime = -1;
    let rafId: number;

    const countFrames = () => {
      const video = videoRef.current;
      if (video && video.currentTime !== lastVideoTime) {
        lastVideoTime = video.currentTime;
        frameCount++;
      }
      const now = performance.now();
      if (now - lastTime >= 1000) {
        setFps(frameCount);
        frameCount = 0;
        lastTime = now;
      }
      rafId = requestAnimationFrame(countFrames);
    };

    rafId = requestAnimationFrame(countFrames);
    return () => cancelAnimationFrame(rafId);
  }, [setFps]);

  const enumerateDevices = useCallback(async (): Promise<VideoSource[]> => {
    try {
      // Request a temporary stream to trigger the macOS permission prompt.
      // WKWebView won't return devices from enumerateDevices() until the
      // user has granted camera access via getUserMedia().
      const tempStream = await navigator.mediaDevices.getUserMedia({ video: true });
      tempStream.getTracks().forEach((t) => t.stop());

      const devices = await navigator.mediaDevices.enumerateDevices();
      return devices
        .filter((d) => d.kind === "videoinput")
        .map((d, i) => ({
          id: d.deviceId,
          label: d.label || `Camera ${i + 1}`,
          type: "local" as const,
          deviceId: d.deviceId,
        }));
    } catch {
      return [];
    }
  }, []);

  return {
    videoRef,
    error,
    enumerateDevices,
  };
}
