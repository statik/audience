import { useEffect, useState } from "react";

const RESET_MS = 3000;

interface ConfirmButtonProps {
  label: string;
  confirmLabel?: string;
  onConfirm: () => void;
  className?: string;
  "aria-label"?: string;
}

/**
 * Two-click destructive action: first click arms ("Confirm?"), second
 * click within 3s fires. Arms back down automatically so a stray click
 * doesn't leave a live trigger behind.
 */
export function ConfirmButton({
  label,
  confirmLabel = "Confirm?",
  onConfirm,
  className,
  "aria-label": ariaLabel,
}: ConfirmButtonProps) {
  const [armed, setArmed] = useState(false);

  useEffect(() => {
    if (!armed) return;
    const timer = setTimeout(() => setArmed(false), RESET_MS);
    return () => clearTimeout(timer);
  }, [armed]);

  return (
    <button
      className={className}
      aria-label={ariaLabel}
      onClick={() => {
        if (armed) {
          setArmed(false);
          onConfirm();
        } else {
          setArmed(true);
        }
      }}
    >
      {armed ? confirmLabel : label}
    </button>
  );
}
