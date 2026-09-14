import type { ReactNode } from 'react';

type LoadingGateProps = {
  active: boolean;
  children: ReactNode;
  message: string;
};

export function LoadingGate({ active, children, message }: LoadingGateProps) {
  return (
    <div
      aria-busy={active}
      className="loading-gate"
      onClickCapture={active ? (event) => { event.preventDefault(); event.stopPropagation(); } : undefined}
    >
      {children}
      {active && <div aria-live="assertive" className="loading-scrim" role="status">
        <div className="loading-card">
          <span aria-hidden="true" className="loading-orb" />
          <strong>{message}</strong>
          <p>Your game save is never opened or changed.</p>
        </div>
      </div>}
    </div>
  );
}
