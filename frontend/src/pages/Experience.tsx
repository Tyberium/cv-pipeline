import { useEffect, useState } from 'react';
import { EmptyState } from '../components/EmptyState';
import type { CareerEvent } from '../types';
import { formatDateRange } from '../utils/dates';
import { BP, cardStyle } from '../theme';

export function Experience() {
  const [experience, setExperience] = useState<CareerEvent[] | null>(null);

  useEffect(() => {
    fetch('/api/experience').then(r => r.json()).then(setExperience);
  }, []);

  if (!experience) return <EmptyState waiting />;
  if (experience.length === 0) return <EmptyState />;

  return (
    <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>

      {/* Page title bar */}
      <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
        <h2 style={{ margin: '0 0 0.2rem', fontSize: '1.5rem', fontWeight: 300, letterSpacing: '0.03em', color: BP.textPrimary }}>
          Employment history
        </h2>
        <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>Most recent first</p>
      </div>

      {/* Timeline */}
      <div style={{ position: 'relative', paddingLeft: '1.25rem' }}>
        {/* Vertical line */}
        <div style={{
          position: 'absolute', left: 4, top: 8, bottom: 0,
          width: 1, background: BP.borderCard,
        }} />

        {experience.map(job => (
          <div key={job.id} style={{ position: 'relative', marginBottom: '1.25rem' }}>
            {/* Timeline dot */}
            <div style={{
              position: 'absolute', left: '-1.1rem', top: 6,
              width: 8, height: 8, borderRadius: '50%',
              background: BP.primary,
              boxShadow: `0 0 6px rgba(97,175,239,0.6)`,
            }} />

            <div style={cardStyle}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '0.5rem', marginBottom: '0.6rem' }}>
                <div>
                  <div style={{ fontSize: '0.875rem', fontWeight: 600, color: BP.textPrimary, marginBottom: '0.1rem' }}>
                    {job.title}
                  </div>
                  <div style={{ fontSize: '0.75rem', color: BP.textMuted, fontStyle: 'italic' }}>
                    {job.company} · {formatDateRange(job.start_date, job.end_date)}
                  </div>
                </div>
                {!job.end_date && (
                  <span style={{
                    background: 'rgba(97,175,239,0.1)', border: `1px solid rgba(97,175,239,0.25)`,
                    color: BP.textAccent, borderRadius: '20px', padding: '0.15rem 0.6rem',
                    fontSize: '0.65rem', fontWeight: 500,
                  }}>
                    Current
                  </span>
                )}
              </div>

              {job.responsibilities.length > 0 && (
                <ul style={{ margin: '0 0 0.5rem', paddingLeft: '1.1rem' }}>
                  {job.responsibilities.map((r, i) => (
                    <li key={i} style={{ fontSize: '0.8125rem', color: BP.textSecondary, marginBottom: '0.2rem', lineHeight: 1.5 }}>
                      {r}
                    </li>
                  ))}
                </ul>
              )}

              {job.key_projects.length > 0 && (
                <>
                  <div style={{
                    fontSize: '0.7rem', fontWeight: 600, color: BP.textAccent,
                    textTransform: 'uppercase', letterSpacing: '0.05em',
                    marginTop: '0.75rem', marginBottom: '0.3rem',
                  }}>
                    Key projects
                  </div>
                  <ul style={{ margin: 0, paddingLeft: '1.1rem' }}>
                    {job.key_projects.map(p => (
                      <li key={p.id} style={{ fontSize: '0.8125rem', color: BP.textSecondary, marginBottom: '0.2rem', lineHeight: 1.5 }}>
                        {p.description}
                      </li>
                    ))}
                  </ul>
                </>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
