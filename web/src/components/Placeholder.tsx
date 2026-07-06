interface PlaceholderProps {
  title: string
  note: string
}

export function Placeholder({ title, note }: PlaceholderProps) {
  return (
    <section className="rounded-lg border border-dashed border-border-strong bg-surface px-4 py-10 text-center">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mt-1 text-sm text-faint">{note}</p>
    </section>
  )
}
