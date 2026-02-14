import { useCallback, useEffect, useRef, useState } from "react";
import { useAppStore } from "../store/app-store";
import type { VideoSource } from "@shared/types";

export function useVideoFeed() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const activeSource = useAppStore((s) => s.activeVideoSource);
  const setIsConnected = useAppStore((s) => s.setIsConnected);
  const setConnectionLabel = useAppStore((s) => s.setConnectionLabel);
  const setFps = useAppStore((s) => s.setFps);
  const [error, setError] = useState<string | null>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const stopCurrentStream = useCallback(() => {
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
    if (videoRef.current) {
      videoRef.current.srcObject = null;
      videoRef.current.src = "";
    }
  }, []);

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
        }
        setIsConnected(true);
        setError(null);
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
      }
      setIsConnected(true);
      setError(null);
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

    if (activeSource.type === "local" && activeSource.deviceId) {
      connectToLocalDevice(activeSource.deviceId);
    } else if (activeSource.type === "ndi" && activeSource.ndiName) {
      // NDI sources use MJPEG stream from backend
      // In production, invoke start_mjpeg_stream and get the port
      const mjpegUrl = `http://127.0.0.1:9090/stream`;
      connectToMjpeg(mjpegUrl);
    }

    return () => {
      stopCurrentStream();
    };
  }, [activeSource, connectToLocalDevice, connectToMjpeg, stopCurrentStream, setIsConnected, setConnectionLabel]);

  // FPS counter
  useEffect(() => {
    if (!videoRef.current) return;
    let lastTime = performance.now();
    let frameCount = 0;

    const countFrame = () => {
      frameCount++;
      const now = performance.now();
      if (now - lastTime >= 1000) {
        setFps(frameCount);
        frameCount = 0;
        lastTime = now;
      }
    };

    const video = videoRef.current;
    video.addEventListener("timeupdate", countFrame);
    return () => {
      video.removeEventListener("timeupdate", countFrame);
    };
  }, [setFps]);

  const enumerateDevices = useCallback(async (): Promise<VideoSource[]> => {
    try {
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
