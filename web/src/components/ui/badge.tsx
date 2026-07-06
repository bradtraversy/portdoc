import type { ReactNode } from 'react'
import { cn } from '../../lib/cn'

export type BadgeVariant = 'default' | 'ok' | 'warn' | 'danger' | 'info'

const variants: Record<BadgeVariant, string> = {
  default: 'text-muted bg-surface-2 border-border-strong',
  ok: 'text-ok bg-ok/10 border-ok/30',
  warn: 'text-warn bg-warn/10 border-warn/30',
  danger: 'text-danger bg-danger/10 border-danger/30',
  info: 'text-info bg-info/10 border-info/30',
}

const dotColors: Record<BadgeVariant, string> = {
  default: 'bg-faint',
  ok: 'bg-ok',
  warn: 'bg-warn',
  danger: 'bg-danger',
  info: 'bg-info',
}

interface BadgeProps {
  variant?: BadgeVariant
  dot?: boolean
  title?: string
  children: ReactNode
}

export function Badge({ variant = 'default', dot = false, title, children }: BadgeProps) {
  return (
    <span
      title={title}
      className={cn(
        'inline-flex items-center gap-1.5 whitespace-nowrap rounded-full border px-2 py-px text-xs font-medium',
        variants[variant],
      )}
    >
      {dot && <span className={cn('size-[7px] flex-none rounded-full', dotColors[variant])} />}
      {children}
    </span>
  )
}
