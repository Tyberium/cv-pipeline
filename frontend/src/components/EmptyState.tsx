import { BP } from '../theme';

interface EmptyStateProps {
  waiting?: boolean;
}

export function EmptyState({ waiting = false }: EmptyStateProps) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 300 }}>
      <div style={{ textAlign: 'center' }}>
        <div style={{ fontFamily: 'monospace', fontSize: '0.875rem', color: BP.textMuted, marginBottom: '0.25rem' }}>
          {waiting ? '$ pipeline running...' : '$ awaiting pipeline run'}
        </div>
        <div style={{ fontSize: '0.875rem', color: BP.textMuted }}>
          {waiting ? 'Data incoming.' : 'Run the container to populate this page.'}
        </div>
      </div>
    </div>
  );
}
