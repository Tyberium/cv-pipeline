import { Stack, Text } from '@mantine/core';
import { useEffect, useState } from 'react';
import type { PipelineRun } from '../types';
import { formatDateTime } from '../utils/dates';
import { BP, cardStyle, cardTitleStyle } from '../theme';

const PIPELINE_STORY = {
  title: 'This CV is a living, breathing pipeline',
  lead: 'What happened was…',
  body: [
    'You ran a single Docker command. Before the browser opened, a script played out in the terminal — decrypt, ingest, transform, bus, warehouse, serve. That wasn\'t a loading spinner. That was the CV.',
    'A single Rust binary unlocks encrypted history and a profile photo with CV_SECRET, pulls live skills and availability from Firebase, normalises everything into PostHog-style capture events, runs them through Redpanda into DuckDB, then serves this site from the gold layer. The wrong secret and the container stops at decrypt; no plaintext ever leaves the image.',
    'The About, Employment, Skills, and Education pages are not hard-coded React props. They are query results from tables this run has just written. The stats below are telemetry from fact_pipeline_run. The startup log at the bottom is the same narrative that scrolled past in your terminal.',
    'I took the problem of "a CV" and totally over-engineered it — to deliberately make something a bit weird, and hopefully a bit more fun.',
    'You can find the source code for this pipeline here: https://github.com/tyberium/cv-pipeline',
  ].join('\n\n'),
};

function toParagraphs(text: string): string[] {
  return text
    .split(/\n\n+/)
    .map(p => p.replace(/\s+/g, ' ').trim())
    .filter(Boolean);
}

function StoryBlock({ text }: { text: string }) {
  return (
    <Stack gap="md" w="100%">
      {toParagraphs(text).map((paragraph, i) => (
        <Text key={i} size="sm" lh={1.75} style={{ color: BP.textSecondary }}>
          {paragraph}
        </Text>
      ))}
    </Stack>
  );
}

function StatChip({ label, value }: { label: string; value: number | string }) {
  return (
    <div style={{
      background: BP.surfaceCard, border: `1px solid ${BP.borderCard}`,
      borderRadius: BP.radiusMd, padding: '0.75rem', textAlign: 'center',
    }}>
      <div style={{ fontSize: '1.5rem', fontWeight: 300, color: BP.primary }}>{value}</div>
      <div style={{ fontSize: '0.6875rem', color: BP.textMuted, marginTop: '0.15rem' }}>{label}</div>
    </div>
  );
}

function Pill({ children, accent }: { children: React.ReactNode; accent?: boolean }) {
  return (
    <span style={{
      background: accent ? 'rgba(97,175,239,0.1)' : 'rgba(255,165,0,0.1)',
      border:     `1px solid ${accent ? 'rgba(97,175,239,0.25)' : 'rgba(255,165,0,0.25)'}`,
      color:      accent ? BP.textAccent : '#ffa500',
      borderRadius: '20px', padding: '0.2rem 0.7rem',
      fontSize: '0.75rem', fontWeight: 500,
    }}>
      {children}
    </span>
  );
}

export function PipelineHealth() {
  const [run, setRun] = useState<PipelineRun | null>(null);
  const [logs, setLogs] = useState<string[] | null>(null);

  useEffect(() => {
    fetch('/api/pipeline').then(r => r.json()).then(setRun);
    fetch('/api/pipeline/logs').then(r => r.json()).then(setLogs);
  }, []);

  const storySection = (
    <div style={cardStyle}>
      <h2 style={{ margin: '0 0 0.5rem', fontSize: '1.35rem', fontWeight: 400, color: BP.textPrimary, lineHeight: 1.3 }}>
        {PIPELINE_STORY.title}
      </h2>
      <Text size="sm" fw={600} mb="sm" style={{ color: BP.textAccent }}>
        {PIPELINE_STORY.lead}
      </Text>
      <StoryBlock text={PIPELINE_STORY.body} />
    </div>
  );

  if (!run) {
    return (
      <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
        {storySection}
        <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
          <h3 style={{ margin: '0 0 0.2rem', fontSize: '1rem', fontWeight: 500, color: BP.textPrimary }}>Health and telemetry</h3>
          <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>Run metrics appear after the pipeline completes</p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 120 }}>
          <span style={{ fontFamily: 'monospace', color: BP.textMuted, fontSize: '0.875rem' }}>$ awaiting pipeline run…</span>
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>

      {storySection}

      <div style={{ borderBottom: `1px solid ${BP.border}`, paddingBottom: '1rem' }}>
        <h3 style={{ margin: '0 0 0.2rem', fontSize: '1rem', fontWeight: 500, color: BP.textPrimary }}>Health and telemetry</h3>
        <p style={{ margin: 0, fontSize: '0.8125rem', color: BP.textMuted }}>
          run_id: {run.run_id} · {formatDateTime(run.ran_at)}
        </p>
      </div>

      {/* Key stats */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '0.75rem' }}>
        <StatChip label="Events Produced" value={run.events_produced} />
        <StatChip label="Events Consumed" value={run.events_consumed} />
        <StatChip label="Duration"        value={`${run.duration_ms}ms`} />
      </div>

      {/* Source breakdown */}
      <div style={cardStyle}>
        <div style={cardTitleStyle}>Sources</div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
          <Pill accent>JSON · {run.source_breakdown.json} events</Pill>
          <Pill>Firebase · {run.source_breakdown.firebase} events</Pill>
        </div>
      </div>

      {/* Event breakdown */}
      <div style={cardStyle}>
        <div style={cardTitleStyle}>Event breakdown</div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
          {Object.entries(run.event_breakdown).map(([type, count]) => (
            <div key={type} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.8125rem', fontFamily: 'monospace', color: BP.textMuted }}>{type}</span>
              <span style={{
                background: 'rgba(97,175,239,0.1)', border: `1px solid rgba(97,175,239,0.2)`,
                color: BP.textAccent, borderRadius: '20px', padding: '0.1rem 0.5rem',
                fontSize: '0.7rem', fontWeight: 600,
              }}>{count}</span>
            </div>
          ))}
        </div>
      </div>

      {logs && (
        <div style={cardStyle}>
          <div style={cardTitleStyle}>Startup log</div>
          <pre style={{
            margin: 0,
            fontFamily: 'monospace',
            fontSize: '0.75rem',
            lineHeight: 1.6,
            color: BP.textSecondary,
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            background: 'rgba(0,0,0,0.3)',
            border: `1px solid ${BP.borderCard}`,
            borderRadius: BP.radiusMd,
            padding: '0.75rem 1rem',
            maxHeight: '28rem',
            overflowY: 'auto',
          }}>
            {logs.map((line, i) => {
              const isCheckmark = line.includes('✓');
              const isHeader = /^\[/.test(line.trim());
              return (
                <div key={i} style={{
                  color: isCheckmark ? '#00cc55' : isHeader ? BP.textAccent : BP.textSecondary,
                  fontWeight: isHeader ? 600 : undefined,
                }}>
                  {line || '\u00a0'}
                </div>
              );
            })}
          </pre>
        </div>
      )}
    </div>
  );
}
