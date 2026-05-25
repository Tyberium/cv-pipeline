// types — API response shapes
//
// These mirror the shapes defined in planning/data_models.md exactly.
// All timestamps arrive as ISO 8601 strings. Do not parse them here.
// Use src/utils/dates.ts for any display formatting.

export interface Availability {
  available: boolean;
  message: string;
}

export interface Profile {
  name: string;
  title: string;
  location: string;
  email: string;
  linkedin: string;
  github: string;
  summary: string;
  about_me: string;
  why_posthog: string;
  outside_work: string;
  availability: Availability;
  interests: string[];
}

export interface KeyProject {
  id: string;
  career_event_id: string;
  description: string;
  sort_order: number;
}

export interface CareerEvent {
  id: string;
  company: string;
  title: string;
  start_date: string;    // ISO date string "YYYY-MM-DD"
  end_date: string | null;
  responsibilities: string[];
  key_projects: KeyProject[];
  source: 'json' | 'firebase';
}

export interface Skill {
  skill: string;
  level: 'expert' | 'proficient' | 'familiar' | null;
}

export type SkillsByCategory = Record<string, Skill[]>;

export interface EducationEntry {
  id: string;
  year: number;
  qualification: string;
  institution: string | null;
  type: 'degree' | 'diploma' | 'certification' | 'course';
  sort_order: number;
}

export interface EventBreakdown {
  profile_update: number;
  career_milestone: number;
  responsibility_added: number;
  project_highlight: number;
  skill_added: number;
  education_entry: number;
}

export interface PipelineRun {
  run_id: string;
  events_produced: number;
  events_consumed: number;
  duration_ms: number;
  ran_at: string;        // ISO timestamp string
  source_breakdown: { json: number; firebase: number };
  event_breakdown: EventBreakdown;
}

