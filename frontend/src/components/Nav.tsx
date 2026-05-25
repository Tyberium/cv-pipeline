import { Link, useLocation } from 'react-router-dom';
import { BP } from '../theme';

const links = [
  {
    to: '/',
    label: 'About',
    subtitle: 'Introduction',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
        <polyline points="9 22 9 12 15 12 15 22"/>
      </svg>
    ),
  },
  {
    to: '/experience',
    label: 'Employment',
    subtitle: 'Work history',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <rect x="2" y="7" width="20" height="14" rx="2"/>
        <path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/>
      </svg>
    ),
  },
  {
    to: '/skills',
    label: 'Skills',
    subtitle: 'Technologies & tools',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <rect x="9" y="9" width="6" height="6"/>
        <rect x="2" y="2" width="20" height="20" rx="2"/>
        <path d="M9 2v7M15 2v7M9 15v7M15 15v7M2 9h7M2 15h7M15 9h7M15 15h7"/>
      </svg>
    ),
  },
  {
    to: '/education',
    label: 'Education',
    subtitle: 'Qualifications & certs',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
        <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
      </svg>
    ),
  },
  {
    to: '/pipeline',
    label: 'Pipeline',
    subtitle: 'Health & telemetry',
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
      </svg>
    ),
  },
];

export function Nav() {
  const { pathname } = useLocation();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>

      {/* Logo + name */}
      <div style={{
        padding: '1rem',
        borderBottom: `1px solid ${BP.border}`,
        display: 'flex',
        alignItems: 'center',
        gap: '0.75rem',
      }}>
        <div style={{
          width: 32, height: 32,
          borderRadius: BP.radiusMd,
          background: `linear-gradient(135deg, ${BP.primary}, ${BP.primaryHover})`,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontWeight: 700, fontSize: '0.9rem', color: '#fff',
          flexShrink: 0,
        }}>
          DC
        </div>
        <div>
          <div style={{ fontSize: '0.9rem', fontWeight: 700, color: BP.textPrimary }}>Dave Carroll</div>
          <div style={{ fontSize: '0.7rem', color: BP.textMuted }}>Data Engineer</div>
        </div>
      </div>

      {/* Nav items */}
      <div style={{ flex: 1, padding: '0.75rem', display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
        {links.map(({ to, label, subtitle, icon }) => {
          const active = pathname === to;
          return (
            <Link
              key={to}
              to={to}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.75rem',
                padding: '0.6rem 0.75rem',
                borderRadius: BP.radiusMd,
                textDecoration: 'none',
                color: BP.textPrimary,
                background: active ? BP.primary : 'transparent',
                transition: 'background 150ms',
              }}
              onMouseEnter={e => { if (!active) (e.currentTarget as HTMLElement).style.background = BP.surfaceHover; }}
              onMouseLeave={e => { if (!active) (e.currentTarget as HTMLElement).style.background = 'transparent'; }}
            >
              <div style={{
                width: 28, height: 28,
                borderRadius: '6px',
                background: active ? 'rgba(255,255,255,0.2)' : 'rgba(97,175,239,0.1)',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                flexShrink: 0,
                color: active ? '#fff' : BP.textAccent,
              }}>
                <div style={{ width: 16, height: 16 }}>{icon}</div>
              </div>
              <div style={{ lineHeight: 1.2 }}>
                <div style={{ fontSize: '0.8125rem', fontWeight: 500 }}>{label}</div>
                <div style={{ fontSize: '0.6875rem', color: active ? 'rgba(255,255,255,0.75)' : BP.textSecondary }}>
                  {subtitle}
                </div>
              </div>
            </Link>
          );
        })}
      </div>

      {/* Footer */}
      <div style={{
        padding: '0.75rem 1rem',
        borderTop: `1px solid ${BP.border}`,
        fontSize: '0.75rem',
        color: BP.textMuted,
        textAlign: 'center',
      }}>
        battleplan.uk / tyberium
      </div>
    </div>
  );
}
