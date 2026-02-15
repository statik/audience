import { useEffect, useRef } from "react";
import { useAppStore } from "../store/app-store";

interface SimulatedFeedProps {
  width: number;
  height: number;
}

const GRID_COLOR = "rgba(42, 42, 74, 0.7)";
const LABEL_COLOR = "rgba(106, 106, 138, 0.8)";
const CROSSHAIR_COLOR = "rgba(224, 224, 255, 0.6)";
const READOUT_COLOR = "rgba(192, 192, 224, 0.9)";
const EDGE_INDICATOR_COLOR = "rgba(58, 58, 90, 0.8)";

/** Grid spacing in normalized units. */
const GRID_STEP = 0.1;

/** Full range of the virtual scene in normalized units per axis direction. */
const SCENE_EXTENT = 1.5;

function drawFrame(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  pan: number,
  tilt: number,
  zoom: number,
) {
  ctx.clearRect(0, 0, width, height);

  // Zoom: at 0 (wide) scale=1, at 1 (telephoto) scale=4
  const scale = 1 + zoom * 3;

  ctx.save();
  ctx.translate(width / 2, height / 2);
  ctx.scale(scale, scale);

  // Pan/tilt offset: map normalized [-1,1] to pixel displacement.
  // At scale=1 the full scene is visible; panning shifts the viewport.
  const baseSpan = Math.min(width, height);
  const offsetX = -pan * baseSpan * 0.5;
  const offsetY = tilt * baseSpan * 0.5;
  ctx.translate(offsetX, offsetY);

  // Draw grid lines
  ctx.strokeStyle = GRID_COLOR;
  ctx.lineWidth = 1 / scale;

  const gridPx = GRID_STEP * baseSpan * 0.5;
  const countH = Math.ceil(SCENE_EXTENT / GRID_STEP);
  const countV = Math.ceil(SCENE_EXTENT / GRID_STEP);

  for (let i = -countH; i <= countH; i++) {
    const x = i * gridPx;
    ctx.beginPath();
    ctx.moveTo(x, -countV * gridPx);
    ctx.lineTo(x, countV * gridPx);
    ctx.stroke();
  }
  for (let j = -countV; j <= countV; j++) {
    const y = j * gridPx;
    ctx.beginPath();
    ctx.moveTo(-countH * gridPx, y);
    ctx.lineTo(countH * gridPx, y);
    ctx.stroke();
  }

  // Draw coordinate labels at grid intersections
  ctx.fillStyle = LABEL_COLOR;
  const fontSize = Math.max(8, 12 / scale);
  ctx.font = `${fontSize}px monospace`;
  ctx.textAlign = "center";
  ctx.textBaseline = "bottom";

  // Label every 5th intersection for readability
  const labelStep = 5;
  for (let i = -countH; i <= countH; i += labelStep) {
    for (let j = -countV; j <= countV; j += labelStep) {
      const nx = (i * GRID_STEP).toFixed(1);
      // Invert Y so positive tilt = up
      const ny = (-(j * GRID_STEP)).toFixed(1);
      ctx.fillText(
        `${nx}, ${ny}`,
        i * gridPx,
        j * gridPx - 2 / scale,
      );
    }
  }

  ctx.restore();

  // Directional edge indicators (drawn in screen space)
  ctx.fillStyle = EDGE_INDICATOR_COLOR;
  ctx.font = "bold 14px monospace";
  ctx.textBaseline = "middle";

  ctx.textAlign = "center";
  ctx.fillText("U", width / 2, 16);
  ctx.fillText("D", width / 2, height - 16);

  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillText("L", 8, height / 2);
  ctx.textAlign = "right";
  ctx.fillText("R", width - 8, height / 2);

  // Crosshair at center
  ctx.strokeStyle = CROSSHAIR_COLOR;
  ctx.lineWidth = 1;
  ctx.globalAlpha = 0.5;

  const chSize = 20;
  const cx = width / 2;
  const cy = height / 2;

  ctx.beginPath();
  ctx.moveTo(cx - chSize, cy);
  ctx.lineTo(cx + chSize, cy);
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(cx, cy - chSize);
  ctx.lineTo(cx, cy + chSize);
  ctx.stroke();

  ctx.globalAlpha = 1;

  // Position readout
  ctx.fillStyle = READOUT_COLOR;
  ctx.font = "12px monospace";
  ctx.textAlign = "left";
  ctx.textBaseline = "top";
  ctx.fillText(
    `P:${pan.toFixed(3)}  T:${tilt.toFixed(3)}  Z:${zoom.toFixed(3)}`,
    8,
    8,
  );
}

export function SimulatedFeed({ width, height }: SimulatedFeedProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const currentPosition = useAppStore((s) => s.currentPosition);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || width <= 0 || height <= 0) return;

    canvas.width = width;
    canvas.height = height;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const frameId = requestAnimationFrame(() => {
      drawFrame(
        ctx,
        width,
        height,
        currentPosition.pan,
        currentPosition.tilt,
        currentPosition.zoom,
      );
    });

    return () => cancelAnimationFrame(frameId);
  }, [width, height, currentPosition]);

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 w-full h-full"
      style={{ imageRendering: "pixelated" }}
    />
  );
}
