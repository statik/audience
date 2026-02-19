import { useCallback, useRef, useState } from "react";
import { usePtzControl } from "../hooks/usePtzControl";

const PAD_SIZE = 160;
const THUMB_RADIUS = 14;
const OUTER_RADIUS = PAD_SIZE / 2;
const THROTTLE_MS = 50;

export function JoystickPad() {
  const { continuousMove, stop } = usePtzControl();
  const containerRef = useRef<HTMLDivElement>(null);
  const lastSendTime = useRef(0);
  const [thumbPos, setThumbPos] = useState({ x: 0, y: 0 });
  const [active, setActive] = useState(false);

  const getOffset = useCallback(
    (clientX: number, clientY: number) => {
      if (!containerRef.current) return { x: 0, y: 0 };
      const rect = containerRef.current.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = clientX - cx;
      const dy = clientY - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const maxDist = OUTER_RADIUS - THUMB_RADIUS;
      if (dist > maxDist) {
        const scale = maxDist / dist;
        return { x: dx * scale, y: dy * scale };
      }
      return { x: dx, y: dy };
    },
    []
  );

  const sendMove = useCallback(
    (x: number, y: number) => {
      const now = Date.now();
      if (now - lastSendTime.current < THROTTLE_MS) return;
      lastSendTime.current = now;
      const maxDist = OUTER_RADIUS - THUMB_RADIUS;
      const panSpeed = x / maxDist;
      const tiltSpeed = -y / maxDist; // screen Y is inverted
      continuousMove(panSpeed, tiltSpeed);
    },
    [continuousMove]
  );

  const handlePointerDown = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      (e.target as HTMLElement).setPointerCapture(e.pointerId);
      setActive(true);
      const offset = getOffset(e.clientX, e.clientY);
      setThumbPos(offset);
      sendMove(offset.x, offset.y);
    },
    [getOffset, sendMove]
  );

  const handlePointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!active) return;
      const offset = getOffset(e.clientX, e.clientY);
      setThumbPos(offset);
      sendMove(offset.x, offset.y);
    },
    [active, getOffset, sendMove]
  );

  const handlePointerUp = useCallback(() => {
    setActive(false);
    setThumbPos({ x: 0, y: 0 });
    stop();
  }, [stop]);

  const center = PAD_SIZE / 2;

  return (
    <div
      ref={containerRef}
      className="relative mx-auto cursor-grab active:cursor-grabbing select-none touch-none"
      style={{ width: PAD_SIZE, height: PAD_SIZE }}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerLeave={handlePointerUp}
    >
      <svg
        width={PAD_SIZE}
        height={PAD_SIZE}
        viewBox={`0 0 ${PAD_SIZE} ${PAD_SIZE}`}
      >
        {/* Outer boundary */}
        <circle
          cx={center}
          cy={center}
          r={OUTER_RADIUS - 2}
          fill="none"
          stroke="var(--color-border)"
          strokeWidth="2"
        />
        {/* Crosshair guides */}
        <line
          x1={center}
          y1={THUMB_RADIUS + 4}
          x2={center}
          y2={PAD_SIZE - THUMB_RADIUS - 4}
          stroke="var(--color-border)"
          strokeWidth="1"
          opacity="0.3"
        />
        <line
          x1={THUMB_RADIUS + 4}
          y1={center}
          x2={PAD_SIZE - THUMB_RADIUS - 4}
          y2={center}
          stroke="var(--color-border)"
          strokeWidth="1"
          opacity="0.3"
        />
        {/* Thumb */}
        <circle
          cx={center + thumbPos.x}
          cy={center + thumbPos.y}
          r={THUMB_RADIUS}
          fill={active ? "var(--color-primary)" : "var(--color-bg-card)"}
          stroke="var(--color-text-muted)"
          strokeWidth="2"
        />
      </svg>
    </div>
  );
}
