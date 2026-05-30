import { Code, List, Stack, Text } from '@mantine/core';
import { useEffect, useState } from 'react';
import type { Profile } from '../types';
import { BP, cardStyle, cardTitleStyle } from '../theme';

/** Paragraphs from `\n\n` in source; soft line breaks collapsed within each block. */
function toParagraphs(text: string): string[] {
  return text
    .split(/\n\n+/)
    .map(p => p.replace(/\s+/g, ' ').trim())
    .filter(Boolean);
}

function CopyBlock({ text }: { text: string }) {
  const paragraphs = toParagraphs(text);
  return (
    <Stack gap="md" w="100%">
      {paragraphs.map((paragraph, i) => (
        <Text key={i} size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
          {paragraph}
        </Text>
      ))}
    </Stack>
  );
}

/** Split highlights into optional section title + bullet lines (• or -). */
function parseHighlights(text: string): { title: string | null; bullets: string[]; prose: string[] } {
  const blocks = toParagraphs(text);
  const bullets: string[] = [];
  const prose: string[] = [];

  for (const block of blocks) {
    if (/^[•\-]\s/.test(block)) {
      bullets.push(block.replace(/^[•\-]\s*/, ''));
    } else {
      prose.push(block);
    }
  }

  const title = bullets.length > 0 && prose.length > 0 ? prose[0] : null;
  const extraProse = title ? prose.slice(1) : prose;

  return { title, bullets, prose: extraProse };
}

function HighlightsBlock({ text }: { text: string }) {
  const { title, bullets, prose } = parseHighlights(text);

  if (bullets.length === 0) {
    return <CopyBlock text={text} />;
  }

  return (
    <Stack gap="md" w="100%">
      {title && (
        <Text size="sm" fw={500} style={{ color: BP.textPrimary }}>
          {title}
        </Text>
      )}
      {prose.map((paragraph, i) => (
        <Text key={i} size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
          {paragraph}
        </Text>
      ))}
      <List
        spacing="sm"
        size="sm"
        styles={{
          item: { color: BP.textSecondary, lineHeight: 1.75 },
          itemWrapper: { alignItems: 'flex-start' },
        }}
      >
        {bullets.map((item, i) => (
          <List.Item key={i}>{item}</List.Item>
        ))}
      </List>
    </Stack>
  );
}

const DOCKER_RUN_RE = /docker run[^\n]+/;
const SOURCE_URL_RE = /^Source:\s*(https?:\/\/\S+)/;

function WhyPostHogBlock({ text }: { text: string }) {
  const paragraphs = toParagraphs(text);

  return (
    <Stack gap="md" w="100%">
      {paragraphs.map((paragraph, i) => {
        if (paragraph === 'So I built one.') {
          return (
            <Text key={i} size="sm" lh={1.75} fw={600} style={{ color: BP.textPrimary }}>
              {paragraph}
            </Text>
          );
        }

        const dockerMatch = paragraph.match(DOCKER_RUN_RE);
        if (dockerMatch) {
          const before = paragraph.slice(0, dockerMatch.index).trim();
          const after = paragraph.slice((dockerMatch.index ?? 0) + dockerMatch[0].length).trim();
          return (
            <Stack key={i} gap="sm">
              {before && (
                <Text size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
                  {before}
                </Text>
              )}
              <Code
                block
                style={{
                  background: 'rgba(0,0,0,0.5)',
                  border: `1px solid ${BP.borderCard}`,
                  color: BP.textAccent,
                  fontSize: '0.75rem',
                  padding: '0.75rem 1rem',
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-all',
                }}
              >
                {dockerMatch[0]}
              </Code>
              {after && (
                <Text size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
                  {after}
                </Text>
              )}
            </Stack>
          );
        }

        const sourceMatch = paragraph.match(SOURCE_URL_RE);
        if (sourceMatch) {
          return (
            <Text key={i} size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
              Source:{' '}
              <a href={sourceMatch[1]} target="_blank" rel="noreferrer" style={{ color: BP.textAccent }}>
                {sourceMatch[1]}
              </a>
            </Text>
          );
        }

        return (
          <Text key={i} size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
            {paragraph}
          </Text>
        );
      })}
    </Stack>
  );
}

function Pill({ children, muted }: { children: React.ReactNode; muted?: boolean }) {
  return (
    <span style={{
      background:   muted ? 'rgba(255,255,255,0.04)' : 'rgba(97,175,239,0.1)',
      border:       muted ? `1px solid ${BP.border}` : `1px solid rgba(97,175,239,0.25)`,
      color:        muted ? BP.textSecondary : BP.textAccent,
      borderRadius: '20px',
      padding:      '0.2rem 0.7rem',
      fontSize:     '0.7rem',
      fontWeight:   500,
    }}>
      {children}
    </span>
  );
}

const posthogCardStyle: React.CSSProperties = {
  ...cardStyle,
  borderLeft: '3px solid rgba(97,175,239,0.6)',
};

export function About() {
  const [profile, setProfile] = useState<Profile | null>(null);

  useEffect(() => {
    fetch('/api/profile').then(r => r.json()).then(setProfile);
  }, []);

  if (!profile) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '80vh' }}>
        <span style={{ fontFamily: 'monospace', color: BP.textMuted, fontSize: '0.875rem' }}>$ loading profile...</span>
      </div>
    );
  }

  return (
    <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>

      <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
        <h2 style={{ margin: '0 0 0.2rem', fontSize: '1.5rem', fontWeight: 300, letterSpacing: '0.03em', color: BP.textPrimary }}>
          About
        </h2>
        <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>Introduction & contact</p>
      </div>

      <div style={cardStyle}>
        <div style={{ display: 'flex', gap: '1.25rem', alignItems: 'flex-start' }}>
          <img
            src="/api/profile/photo"
            alt={profile.name}
            width={88}
            height={88}
            style={{
              width: 88,
              height: 88,
              borderRadius: BP.radiusLg,
              border: '2px solid rgba(97,175,239,0.4)',
              objectFit: 'cover',
              objectPosition: 'center top',
              flexShrink: 0,
              boxShadow: '0 0 16px rgba(97,175,239,0.2)',
            }}
          />
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: '1.6rem', fontWeight: 300, letterSpacing: '0.04em', marginBottom: '0.15rem', color: BP.textPrimary }}>
              {profile.name}
            </div>
            <div style={{ fontSize: '1rem', color: BP.textAccent, marginBottom: '0.75rem' }}>
              {profile.title}
            </div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem', marginBottom: '0.75rem' }}>
              <Pill>{profile.email}</Pill>
              <Pill>{profile.location}</Pill>
              {profile.linkedin && (
                <a href={profile.linkedin} target="_blank" rel="noreferrer" style={{ textDecoration: 'none' }}>
                  <Pill muted>LinkedIn</Pill>
                </a>
              )}
              {profile.github && (
                <a href={profile.github} target="_blank" rel="noreferrer" style={{ textDecoration: 'none' }}>
                  <Pill muted>GitHub</Pill>
                </a>
              )}
            </div>
            <div style={{ borderTop: `1px solid ${BP.border}`, paddingTop: '0.65rem' }}>
              <span style={{ fontSize: '0.6875rem', color: BP.textMuted, marginRight: '0.5rem', letterSpacing: '0.04em', textTransform: 'uppercase' }}>
                This CV:
              </span>
              {(['Rust', 'TypeScript', 'Axum', 'DuckDB', 'Redpanda'] as const).map(tech => (
                <span key={tech} style={{
                  display: 'inline-block',
                  marginRight: '0.35rem',
                  background: (tech === 'Rust' || tech === 'TypeScript')
                    ? 'rgba(97,175,239,0.15)'
                    : 'rgba(255,165,0,0.08)',
                  border: `1px solid ${(tech === 'Rust' || tech === 'TypeScript')
                    ? 'rgba(97,175,239,0.4)'
                    : 'rgba(255,165,0,0.2)'}`,
                  color: (tech === 'Rust' || tech === 'TypeScript') ? BP.textAccent : '#ffa500',
                  borderRadius: '20px',
                  padding: '0.15rem 0.6rem',
                  fontSize: '0.7rem',
                  fontWeight: (tech === 'Rust' || tech === 'TypeScript') ? 700 : 500,
                }}>
                  {tech}
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>

      {profile.summary && (
        <div style={cardStyle}>
          <div style={cardTitleStyle}>Professional Summary</div>
          <CopyBlock text={profile.summary} />
        </div>
      )}

      {profile.about_me && (
        <div style={cardStyle}>
          <div style={cardTitleStyle}>Highlights</div>
          <HighlightsBlock text={profile.about_me} />
        </div>
      )}

      {(profile.outside_work || profile.interests.length > 0) && (
        <div style={cardStyle}>
          <div style={cardTitleStyle}>Off duty</div>
          {profile.outside_work && (
            <Text size="sm" lh={1.75} mb={profile.interests.length > 0 ? 'md' : 0} style={{ color: BP.textSecondary }}>
              {profile.outside_work.replace(/\s+/g, ' ').trim()}
            </Text>
          )}
          {profile.interests.length > 0 && (
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
              {profile.interests.map(label => (
                <Pill key={label} muted>{label}</Pill>
              ))}
            </div>
          )}
        </div>
      )}

      {profile.why_posthog && (
        <div style={posthogCardStyle}>
          <div style={cardTitleStyle}>Why I'm applying to PostHog</div>
          <WhyPostHogBlock text={profile.why_posthog} />
        </div>
      )}

    </div>
  );
}
