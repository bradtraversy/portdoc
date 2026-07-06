import type { ButtonHTMLAttributes } from 'react'
import { cn } from '../../lib/cn'

type ButtonVariant = 'default' | 'accent' | 'ghost'

const variants: Record<ButtonVariant, string> = {
  default: 'bg-surface-2 text-text border-border-strong hover:border-faint',
  accent: 'bg-accent text-accent-ink border-transparent font-semibold hover:bg-accent-hover',
  ghost: 'bg-transparent text-muted border-transparent hover:text-text hover:border-border-strong',
}

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: 'sm' | 'md'
}

export function Button({ variant = 'default', size = 'md', className, ...props }: ButtonProps) {
  return (
    <button
      className={cn(
        'inline-flex cursor-pointer items-center gap-1.5 rounded-[5px] border font-medium',
        size === 'md' ? 'px-3 py-[5px] text-sm' : 'px-2 py-[3px] text-xs',
        variants[variant],
        'disabled:cursor-not-allowed disabled:opacity-45',
        className,
      )}
      {...props}
    />
  )
}
