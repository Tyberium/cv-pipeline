import { useEffect, useState } from 'react';
import { EmptyState } from '../components/EmptyState';
import type { Skill, SkillsByCategory } from '../types';
import { BP, cardStyle, cardTitleStyle } from '../theme';

function SkillPill({ skill }: { skill: Skill }) {
  // expert = full accent, proficient = muted accent, familiar = muted grey
  const styles: React.CSSProperties = skill.level === 'expert'
    ? { background: 'rgba(97,175,239,0.18)', border: `1px solid rgba(97,175,239,0.4)`, color: BP.textAccent }
    : skill.level === 'proficient'
    ? { background: 'rgba(97,175,239,0.08)', border: `1px solid rgba(97,175,239,0.2)`, color: BP.textAccent }
    : { background: 'rgba(255,255,255,0.04)', border: `1px solid ${BP.border}`, color: BP.textSecondary };

  return (
    <span style={{ ...styles, borderRadius: '20px', padding: '0.2rem 0.7rem', fontSize: '0.7rem', fontWeight: 500 }}>
      {skill.skill}
    </span>
  );
}

export function Skills() {
  const [skills, setSkills] = useState<SkillsByCategory | null>(null);

  useEffect(() => {
    fetch('/api/skills').then(r => r.json()).then(setSkills);
  }, []);

  if (!skills) return <EmptyState waiting />;
  if (Object.keys(skills).length === 0) return <EmptyState />;

  return (
    <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>

      {/* Page title bar */}
      <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
        <h2 style={{ margin: '0 0 0.2rem', fontSize: '1.5rem', fontWeight: 300, letterSpacing: '0.03em', color: BP.textPrimary }}>
          Skills
        </h2>
        <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>Technologies & tools</p>
      </div>

      {/* 2-col grid of category cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '0.75rem' }}>
        {Object.entries(skills).map(([category, items]) => (
          <div key={category} style={cardStyle}>
            <div style={cardTitleStyle}>{category}</div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.35rem' }}>
              {items.map(skill => <SkillPill key={skill.skill} skill={skill} />)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
