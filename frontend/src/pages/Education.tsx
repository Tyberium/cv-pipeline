import { useEffect, useState } from 'react';
import { EmptyState } from '../components/EmptyState';
import type { EducationEntry } from '../types';
import { BP, cardStyle, cardTitleStyle } from '../theme';

function EduRow({ entry }: { entry: EducationEntry }) {
  return (
    <div style={{
      display: 'grid', gridTemplateColumns: '3rem 1fr',
      gap: '0.75rem', padding: '0.4rem 0',
      borderBottom: `1px solid ${BP.border}`,
      fontSize: '0.8125rem', alignItems: 'baseline',
    }}>
      <span style={{ color: BP.textAccent, fontSize: '0.7rem', fontWeight: 600 }}>{entry.year}</span>
      <div>
        <span style={{ color: BP.textPrimary }}>{entry.qualification}</span>
        {entry.institution && (
          <span style={{ color: BP.textMuted }}> · {entry.institution}</span>
        )}
      </div>
    </div>
  );
}

export function Education() {
  const [education, setEducation] = useState<EducationEntry[] | null>(null);

  useEffect(() => {
    fetch('/api/education').then(r => r.json()).then(setEducation);
  }, []);

  if (!education) return <EmptyState waiting />;
  if (education.length === 0) return <EmptyState />;

  const formal  = education.filter(e => e.type === 'degree' || e.type === 'diploma');
  const certs   = education.filter(e => e.type === 'certification');

  return (
    <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>

      {/* Page title bar */}
      <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
        <h2 style={{ margin: '0 0 0.2rem', fontSize: '1.5rem', fontWeight: 300, letterSpacing: '0.03em', color: BP.textPrimary }}>
          Education
        </h2>
        <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>Qualifications & professional certifications</p>
      </div>

      {/* Two-column layout */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: '0.75rem', alignItems: 'start' }}>

        {formal.length > 0 && (
          <div style={cardStyle}>
            <div style={cardTitleStyle}>Formal education</div>
            {formal.map(e => <EduRow key={e.id} entry={e} />)}
          </div>
        )}

        {certs.length > 0 && (
          <div style={cardStyle}>
            <div style={cardTitleStyle}>Certifications & courses</div>
            {certs.map(e => <EduRow key={e.id} entry={e} />)}
          </div>
        )}

      </div>
    </div>
  );
}
