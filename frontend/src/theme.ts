// theme — BattlePlan design system tokens
//
// Pulled directly from battleplan/tokens.ts — dark background, blue accent (#61afef).
// No hex values anywhere else in the frontend; import BP_* constants instead.

import { createTheme } from '@mantine/core';

// Shared surface/colour constants — import these in components instead of hardcoding.
export const BP = {
  primary:      '#61afef',
  primaryHover: '#4a90e2',
  bgMain:       'linear-gradient(135deg, #0a0a0a 0%, #1a1a1a 50%, #0f0f0f 100%)',
  surfaceCard:  'rgba(0,0,0,0.6)',
  surfaceNav:   'rgba(0,0,0,0.8)',
  surfaceHover: 'rgba(97,175,239,0.08)',
  surfaceRaise: 'rgba(255,255,255,0.02)',
  border:       'rgba(255,255,255,0.1)',
  borderCard:   'rgba(97,175,239,0.2)',
  textPrimary:  '#ffffff',
  textSecondary:'#e0e0e0',
  textMuted:    '#a0a0a0',
  textAccent:   '#61afef',
  shadowGlow:   '0 0 20px rgba(97,175,239,0.3)',
  shadowCard:   '0 4px 6px rgba(0,0,0,0.4)',
  radiusSm:     '4px',
  radiusMd:     '8px',
  radiusLg:     '12px',
} as const;

// Reusable inline style for the glassy dark card
export const cardStyle: React.CSSProperties = {
  background:   BP.surfaceCard,
  border:       `1px solid ${BP.borderCard}`,
  borderRadius: BP.radiusLg,
  padding:      '1.1rem 1.25rem',
};

// Card section title — small, uppercase, accent colour
export const cardTitleStyle: React.CSSProperties = {
  fontSize:      '0.75rem',
  fontWeight:    600,
  letterSpacing: '0.06em',
  textTransform: 'uppercase',
  color:         BP.textAccent,
  margin:        '0 0 0.75rem',
};

export const theme = createTheme({
  primaryColor: 'bp',

  colors: {
    // BattlePlan blue — 10-shade array, primary at index 5
    bp: [
      '#ebf5fd',
      '#d0e9f9',
      '#aed4f5',
      '#87bff0',
      '#70b3ed',
      '#61afef',  // [5] primary
      '#4a90e2',  // [6] hover
      '#3a7bc8',
      '#2a65ae',
      '#1a4f94',
    ],
  },

  fontFamily: "'Inter', 'Segoe UI', sans-serif",

  fontSizes: {
    xs: '0.75rem',
    sm: '0.8125rem',
    md: '0.875rem',
    lg: '1rem',
    xl: '1.125rem',
  },

  headings: {
    fontFamily: "'Inter', 'Segoe UI', sans-serif",
    sizes: {
      h1: { fontSize: '1.6rem',  fontWeight: '300' },
      h2: { fontSize: '1.5rem',  fontWeight: '300' },
      h3: { fontSize: '1rem',    fontWeight: '600' },
      h4: { fontSize: '0.75rem', fontWeight: '600' },
    },
  },

  radius: {
    xs: '2px',
    sm: '4px',
    md: '8px',
    lg: '12px',
    xl: '20px',
  },
});
