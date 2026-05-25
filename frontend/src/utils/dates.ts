// dates — the only place in the frontend that touches date formatting
//
// All API responses carry ISO strings. They stay as strings until display.
// new Date() is called here and only here.

export function formatMonthYear(iso: string): string {
  return new Date(iso).toLocaleDateString('en-GB', { month: 'long', year: 'numeric' });
}

export function formatYear(iso: string): string {
  return String(new Date(iso).getFullYear());
}

export function formatDateRange(start: string, end: string | null): string {
  return `${formatMonthYear(start)} – ${end ? formatMonthYear(end) : 'Present'}`;
}

export function formatDateTime(iso: string): string {
  return new Date(iso).toLocaleString('en-GB', {
    day: 'numeric', month: 'short', year: 'numeric',
    hour: '2-digit', minute: '2-digit',
  });
}
