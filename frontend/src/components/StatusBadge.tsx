import { CheckCircle2, AlertTriangle, XCircle, Clock, ShieldAlert } from 'lucide-react';

interface StatusBadgeProps {
  label: string;
  className?: string;
}

export default function StatusBadge({ label, className = '' }: StatusBadgeProps) {
  const normalised = label.toUpperCase().replace(/[^A-Z_]/g, '_');

  let tone = 'badge-neutral';
  let Icon = Clock;

  if (
    normalised.includes('ERROR') ||
    normalised.includes('REJECTED') ||
    normalised.includes('OVER') ||
    normalised.includes('FAILED')
  ) {
    tone = 'badge-danger';
    Icon = XCircle;
  } else if (normalised.includes('DUPLICATE')) {
    tone = 'badge-danger';
    Icon = ShieldAlert;
  } else if (normalised.includes('REVIEW') || normalised.includes('NEEDS_REVIEW')) {
    tone = 'badge-review';
    Icon = Clock;
  } else if (normalised.includes('WARN')) {
    tone = 'badge-warn';
    Icon = AlertTriangle;
  } else if (
    normalised.includes('ACCEPTED') ||
    normalised.includes('COMMITTED') ||
    normalised.includes('RESOLVED') ||
    normalised.includes('ACTIVE') ||
    normalised.includes('VERIFIED')
  ) {
    tone = 'badge-ok';
    Icon = CheckCircle2;
  }

  return (
    <span className={`badge ${tone} ${className}`}>
      <Icon className="w-3.5 h-3.5" aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}
